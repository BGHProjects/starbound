package db

import (
	"encoding/json"
	"fmt"
	"os"
	"sync"
	"time"

	"github.com/BGHProjects/starbound/gateway/internal/models"
	"github.com/google/uuid"
)

type OrderStore struct {
    mu       sync.RWMutex
    filePath string
    orders   []models.Order
}

func NewOrderStore() (*OrderStore, error) {
    return newOrderStore("internal/db/orders.json")
}

func NewTestOrderStore() (*OrderStore, error) {
    tmp, err := os.CreateTemp("", "starbound_test_orders_*.json")
    if err != nil {
        return nil, err
    }
    if _, err := tmp.WriteString("[]"); err != nil {
        return nil, err
    }
    tmp.Close()
    return newOrderStore(tmp.Name())
}

func newOrderStore(path string) (*OrderStore, error) {
    store := &OrderStore{filePath: path}
    if err := store.load(); err != nil {
        return nil, err
    }
    return store, nil
}

func (s *OrderStore) load() error {
    data, err := os.ReadFile(s.filePath)
    if err != nil {
        return fmt.Errorf("failed to read orders file: %w", err)
    }
    return json.Unmarshal(data, &s.orders)
}

func (s *OrderStore) save() error {
    data, err := json.MarshalIndent(s.orders, "", "  ")
    if err != nil {
        return fmt.Errorf("failed to marshal orders: %w", err)
    }
    return os.WriteFile(s.filePath, data, 0644)
}

// GetOrdersByUserID returns all orders for a given user, paginated
func (s *OrderStore) GetOrdersByUserID(userID string, page, limit int) (models.OrderListResponse, error) {
    s.mu.RLock()
    defer s.mu.RUnlock()

    if page <= 0 {
        page = 1
    }
    if limit <= 0 || limit > 100 {
        limit = 20
    }

    var userOrders []models.Order
    for _, o := range s.orders {
        if o.UserID == userID {
            userOrders = append(userOrders, o)
        }
    }

    total := len(userOrders)
    start := (page - 1) * limit
    end := start + limit

    if start >= total {
        return models.OrderListResponse{
            Data:  []models.Order{},
            Total: total,
            Page:  page,
            Limit: limit,
        }, nil
    }
    if end > total {
        end = total
    }

    return models.OrderListResponse{
        Data:  userOrders[start:end],
        Total: total,
        Page:  page,
        Limit: limit,
    }, nil
}

// GetOrderByID returns a single order by ID
func (s *OrderStore) GetOrderByID(id string) (*models.Order, error) {
    s.mu.RLock()
    defer s.mu.RUnlock()

    for i, o := range s.orders {
        if o.ID == id {
            return &s.orders[i], nil
        }
    }
    return nil, fmt.Errorf("order not found: %s", id)
}

// CreateOrder builds and persists a new order from a request
func (s *OrderStore) CreateOrder(userID string, req models.CreateOrderRequest, productDB DB) (*models.Order, error) {
    s.mu.Lock()
    defer s.mu.Unlock()

    now := time.Now().UTC().Format(time.RFC3339)

    // Build line items by looking up each product
    var items []models.OrderItem
    var subtotal float64

    for _, reqItem := range req.Items {
        product, err := productDB.GetProductByID(reqItem.ProductID)
        if err != nil {
            return nil, fmt.Errorf("product not found: %s", reqItem.ProductID)
        }
        if !product.InStock {
            return nil, fmt.Errorf("product out of stock: %s", product.Name)
        }

        lineTotal := product.Price * float64(reqItem.Quantity)
        subtotal += lineTotal

        items = append(items, models.OrderItem{
            ProductID:   product.ID,
            ProductName: product.Name,
            ProductType: string(product.Type),
            Quantity:    reqItem.Quantity,
            UnitPrice:   product.Price,
            LineTotal:   lineTotal,
        })
    }

    // Flat rate shipping — can be made dynamic later
    shippingCost := 2500.00

    order := models.Order{
        ID:              uuid.New().String(),
        UserID:          userID,
        Status:          models.StatusPending,
        Items:           items,
        ShippingAddress: req.ShippingAddress,
        Subtotal:        subtotal,
        ShippingCost:    shippingCost,
        Total:           subtotal + shippingCost,
        Notes:           req.Notes,
        CreatedAt:       now,
        UpdatedAt:       now,
    }

    s.orders = append(s.orders, order)

    if err := s.save(); err != nil {
        return nil, fmt.Errorf("failed to save order: %w", err)
    }

    return &order, nil
}

// CancelOrder sets an order's status to cancelled if it is cancellable
func (s *OrderStore) CancelOrder(id, userID string) (*models.Order, error) {
    s.mu.Lock()
    defer s.mu.Unlock()

    for i, o := range s.orders {
        if o.ID != id {
            continue
        }
        // Ensure the order belongs to the requesting user
        if o.UserID != userID {
            return nil, fmt.Errorf("forbidden")
        }
        if !o.Status.IsCancellable() {
            return nil, fmt.Errorf("order cannot be cancelled from status: %s", o.Status)
        }

        s.orders[i].Status    = models.StatusCancelled
        s.orders[i].UpdatedAt = time.Now().UTC().Format(time.RFC3339)

        if err := s.save(); err != nil {
            return nil, fmt.Errorf("failed to save order: %w", err)
        }

        return &s.orders[i], nil
    }

    return nil, fmt.Errorf("order not found: %s", id)
}