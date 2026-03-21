package models

// OrderStatus represents the current state of an order
type OrderStatus string

const (
    StatusPending           OrderStatus = "pending"
    StatusPaymentProcessing OrderStatus = "payment_processing"
    StatusPaymentFailed     OrderStatus = "payment_failed"
    StatusConfirmed         OrderStatus = "confirmed"
    StatusPreparing         OrderStatus = "preparing"
    StatusShipped           OrderStatus = "shipped"
    StatusInTransit         OrderStatus = "in_transit"
    StatusDelivered         OrderStatus = "delivered"
    StatusCancelled         OrderStatus = "cancelled"
    StatusRefundPending     OrderStatus = "refund_pending"
    StatusRefunded          OrderStatus = "refunded"
)

// cancellableStatuses are the only statuses an order can be cancelled from
var cancellableStatuses = map[OrderStatus]bool{
    StatusPending:           true,
    StatusPaymentProcessing: true,
    StatusPaymentFailed:     true,
    StatusConfirmed:         true,
}

// IsCancellable returns true if the order can be cancelled
func (s OrderStatus) IsCancellable() bool {
    return cancellableStatuses[s]
}

// OrderItem is a single line item within an order
type OrderItem struct {
    ProductID   string  `json:"product_id"`
    ProductName string  `json:"product_name"`
	ImageURL   string  `json:"image_url"`
    ProductType string  `json:"product_type"`
    Quantity    int     `json:"quantity"`
    UnitPrice   float64 `json:"unit_price"`
    LineTotal   float64 `json:"line_total"`
}

// ShippingAddress holds the delivery destination
type ShippingAddress struct {
    FacilityName string `json:"facility_name"`
    SiteCode     string `json:"site_code"`
    AddressLine1 string `json:"address_line_1"          binding:"required"`
    AddressLine2 string `json:"address_line_2,omitempty"`
    City         string `json:"city"                    binding:"required"`
    Country      string `json:"country"                 binding:"required"`
    PostalCode   string `json:"postal_code"             binding:"required"`
}

// Order is the full order record
type Order struct {
    ID              string          `json:"id"`
    UserID          string          `json:"user_id"`
    Status          OrderStatus     `json:"status"`
    Items           []OrderItem     `json:"items"`
    ShippingAddress ShippingAddress `json:"shipping_address"`
    Subtotal        float64         `json:"subtotal"`
    ShippingCost    float64         `json:"shipping_cost"`
    Total           float64         `json:"total"`
    Notes           string          `json:"notes,omitempty"`
    CreatedAt       string          `json:"created_at"`
    UpdatedAt       string          `json:"updated_at"`
}

// CreateOrderRequest is the body for POST /api/orders
type CreateOrderRequest struct {
    Items           []CreateOrderItem `json:"items"            binding:"required,min=1,dive"`
    ShippingAddress ShippingAddress   `json:"shipping_address" binding:"required"`
    Notes           string            `json:"notes"`
}

// CreateOrderItem is a single item in a create order request
type CreateOrderItem struct {
    ProductID string `json:"product_id" binding:"required"`
    Quantity  int    `json:"quantity"   binding:"required,min=1"`
}

// OrderListResponse wraps a paginated list of orders
type OrderListResponse struct {
    Data  []Order `json:"data"`
    Total int     `json:"total"`
    Page  int     `json:"page"`
    Limit int     `json:"limit"`
}