package handlers

import (
	"net/http"
	"strconv"

	"github.com/BGHProjects/starbound/gateway/internal/db"
	"github.com/BGHProjects/starbound/gateway/internal/models"
	"github.com/gin-gonic/gin"
)

type OrderHandler struct {
    orders   *db.OrderStore
    products db.DB
}

func NewOrderHandler(orders *db.OrderStore, products db.DB) *OrderHandler {
    return &OrderHandler{orders: orders, products: products}
}

// GetOrders godoc
// @Summary      List current user's orders
// @Description  Returns a paginated list of all orders for the authenticated user
// @Tags         orders
// @Produce      json
// @Security     BearerAuth
// @Param        page   query     int  false  "Page number (default 1)"
// @Param        limit  query     int  false  "Items per page (default 20)"
// @Success      200    {object}  models.OrderListResponse
// @Failure      401    {object}  map[string]string
// @Failure      500    {object}  map[string]string
// @Router       /orders [get]
func (h *OrderHandler) GetOrders(c *gin.Context) {
    userID := c.GetString("user_id")

    page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
    limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))

    result, err := h.orders.GetOrdersByUserID(userID, page, limit)
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{
            "error": "failed to retrieve orders",
        })
        return
    }

    c.JSON(http.StatusOK, result)
}

// GetOrderByID godoc
// @Summary      Get a single order
// @Description  Returns a single order by ID — user can only access their own orders
// @Tags         orders
// @Produce      json
// @Security     BearerAuth
// @Param        id   path      string  true  "Order ID"
// @Success      200  {object}  models.Order
// @Failure      401  {object}  map[string]string
// @Failure      403  {object}  map[string]string
// @Failure      404  {object}  map[string]string
// @Router       /orders/{id} [get]
func (h *OrderHandler) GetOrderByID(c *gin.Context) {
    userID := c.GetString("user_id")
    id     := c.Param("id")

    order, err := h.orders.GetOrderByID(id)
    if err != nil {
        c.JSON(http.StatusNotFound, gin.H{"error": "order not found"})
        return
    }

    // Users can only view their own orders
    if order.UserID != userID {
        c.JSON(http.StatusForbidden, gin.H{"error": "access denied"})
        return
    }

    c.JSON(http.StatusOK, order)
}

// CreateOrder godoc
// @Summary      Create a new order
// @Description  Creates a new order for the authenticated user
// @Tags         orders
// @Accept       json
// @Produce      json
// @Security     BearerAuth
// @Param        body  body      models.CreateOrderRequest  true  "Order details"
// @Success      201   {object}  models.Order
// @Failure      400   {object}  map[string]string
// @Failure      401   {object}  map[string]string
// @Failure      422   {object}  map[string]string
// @Failure      500   {object}  map[string]string
// @Router       /orders [post]
func (h *OrderHandler) CreateOrder(c *gin.Context) {
    userID := c.GetString("user_id")

    var req models.CreateOrderRequest
    if err := c.ShouldBindJSON(&req); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
        return
    }

    order, err := h.orders.CreateOrder(userID, req, h.products)
    if err != nil {
        errMsg := err.Error()
        // Product not found or out of stock — unprocessable
        if len(errMsg) > 0 {
            c.JSON(http.StatusUnprocessableEntity, gin.H{"error": errMsg})
            return
        }
        c.JSON(http.StatusInternalServerError, gin.H{
            "error": "failed to create order",
        })
        return
    }

    c.JSON(http.StatusCreated, order)
}

// CancelOrder godoc
// @Summary      Cancel an order
// @Description  Cancels a pending order — only works for cancellable statuses
// @Tags         orders
// @Produce      json
// @Security     BearerAuth
// @Param        id   path      string  true  "Order ID"
// @Success      200  {object}  models.Order
// @Failure      400  {object}  map[string]string
// @Failure      401  {object}  map[string]string
// @Failure      403  {object}  map[string]string
// @Failure      404  {object}  map[string]string
// @Router       /orders/{id}/cancel [put]
func (h *OrderHandler) CancelOrder(c *gin.Context) {
    userID := c.GetString("user_id")
    id     := c.Param("id")

    order, err := h.orders.CancelOrder(id, userID)
    if err != nil {
        switch err.Error() {
        case "forbidden":
            c.JSON(http.StatusForbidden, gin.H{"error": "access denied"})
        case "order not found: " + id:
            c.JSON(http.StatusNotFound, gin.H{"error": "order not found"})
        default:
            c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
        }
        return
    }

    c.JSON(http.StatusOK, order)
}