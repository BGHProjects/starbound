package db

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"

	"github.com/BGHProjects/starbound/gateway/internal/models"
)

// DB is the database client interface.
// Currently backed by a local JSON file — swap in Supabase later
// by implementing this interface with a real client.
type DB interface {
    GetProducts(filters models.ProductFilters) (models.ProductListResponse, error)
    GetProductByID(id string) (*models.Product, error)
}

// JSONFileDB implements DB using a local JSON seed file
type JSONFileDB struct {
    products []models.Product
}

// NewDB creates a new database client.
// Reads from local JSON file for now.
func NewDB() (DB, error) {
    return newJSONFileDB("internal/db/products_seed.json")
}

func newJSONFileDB(path string) (*JSONFileDB, error) {
    data, err := os.ReadFile(path)
    if err != nil {
        return nil, fmt.Errorf("failed to read seed file: %w", err)
    }

    var products []models.Product
    if err := json.Unmarshal(data, &products); err != nil {
        return nil, fmt.Errorf("failed to parse seed file: %w", err)
    }

    return &JSONFileDB{products: products}, nil
}

// GetProducts returns a filtered, paginated list of products
func (db *JSONFileDB) GetProducts(filters models.ProductFilters) (models.ProductListResponse, error) {
    // Set sensible defaults for pagination
    if filters.Page <= 0 {
        filters.Page = 1
    }
    if filters.Limit <= 0 || filters.Limit > 100 {
        filters.Limit = 20
    }

    // Filter
    var filtered []models.Product
    for _, p := range db.products {
        if filters.Group != "" && p.Group != filters.Group {
            continue
        }
        if filters.Type != "" && p.Type != filters.Type {
            continue
        }
        if filters.Search != "" {
            search := strings.ToLower(filters.Search)
            if !strings.Contains(strings.ToLower(p.Name), search) {
                continue
            }
        }
        filtered = append(filtered, p)
    }

    total := len(filtered)

    // Paginate
    start := (filters.Page - 1) * filters.Limit
    end := start + filters.Limit
    if start >= total {
        return models.ProductListResponse{
            Data:  []models.ProductListItem{},
            Total: total,
            Page:  filters.Page,
            Limit: filters.Limit,
        }, nil
    }
    if end > total {
        end = total
    }

    // Map to lightweight list items
    items := make([]models.ProductListItem, 0, end-start)
    for _, p := range filtered[start:end] {
        items = append(items, models.ProductListItem{
            ID:         p.ID,
            Name:       p.Name,
            Group:      p.Group,
            Type:       p.Type,
            Price:      p.Price,
            ImageURL:   p.ImageURL,
            InStock:    p.InStock,
            StockCount: p.StockCount,
        })
    }

    return models.ProductListResponse{
        Data:  items,
        Total: total,
        Page:  filters.Page,
        Limit: filters.Limit,
    }, nil
}

// GetProductByID returns a single product with full attributes
func (db *JSONFileDB) GetProductByID(id string) (*models.Product, error) {
    for _, p := range db.products {
        if p.ID == id {
            return &p, nil
        }
    }
    return nil, fmt.Errorf("product not found: %s", id)
}

// NewTestDB creates a DB instance pointing at the seed file
// for use in tests — path is relative to the gateway root
func NewTestDB() (DB, error) {
    return newJSONFileDB("../internal/db/products_seed.json")
}