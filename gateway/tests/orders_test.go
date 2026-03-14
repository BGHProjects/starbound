package tests

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/BGHProjects/starbound/gateway/internal/db"
	"github.com/BGHProjects/starbound/gateway/internal/handlers"
	"github.com/BGHProjects/starbound/gateway/internal/middleware"
	"github.com/BGHProjects/starbound/gateway/internal/models"
	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// setupOrderRouter creates a test router with all order and auth routes
func setupOrderRouter() (*gin.Engine, *db.UserStore) {
    gin.SetMode(gin.TestMode)

    productDB, err := db.NewTestDB()
    if err != nil {
        panic("failed to initialise test product db: " + err.Error())
    }

    userStore, err := db.NewTestUserStore()
    if err != nil {
        panic("failed to initialise test user store: " + err.Error())
    }

    orderStore, err := db.NewTestOrderStore()
    if err != nil {
        panic("failed to initialise test order store: " + err.Error())
    }

    productHandler := handlers.NewProductHandler(productDB)
    authHandler    := handlers.NewAuthHandler(userStore)
    orderHandler   := handlers.NewOrderHandler(orderStore, productDB)

    r := gin.New()

    api := r.Group("/api")
    {
        auth := api.Group("/auth")
        {
            auth.POST("/register", authHandler.Register)
            auth.POST("/login",    authHandler.Login)
        }

        _ = productHandler

        orders := api.Group("/orders", middleware.RequireAuth())
        {
            orders.GET("",            orderHandler.GetOrders)
            orders.POST("",           orderHandler.CreateOrder)
            orders.GET("/:id",        orderHandler.GetOrderByID)
            orders.PUT("/:id/cancel", orderHandler.CancelOrder)
        }
    }

    return r, userStore
}

// registerAndLogin is a helper that creates a user and returns a JWT token
func registerAndLogin(t *testing.T, r *gin.Engine, email string) string {
    // Register
    postJSON(r, "/api/auth/register", map[string]string{
        "email":    email,
        "name":     "Test User",
        "password": "password123",
    })

    // Login and extract token
    w := postJSON(r, "/api/auth/login", map[string]string{
        "email":    email,
        "password": "password123",
    })

    var resp models.AuthResponse
    err := json.Unmarshal(w.Body.Bytes(), &resp)
    require.NoError(t, err)
    require.NotEmpty(t, resp.Token)

    return resp.Token
}

// authRequest is a helper that makes an authenticated request
func authRequest(r *gin.Engine, method, path, token string, body interface{}) *httptest.ResponseRecorder {
    var req *http.Request
    if body != nil {
        b, _ := json.Marshal(body)
        req, _ = http.NewRequest(method, path, bytes.NewBuffer(b))
        req.Header.Set("Content-Type", "application/json")
    } else {
        req, _ = http.NewRequest(method, path, nil)
    }
    req.Header.Set("Authorization", "Bearer "+token)

    w := httptest.NewRecorder()
    r.ServeHTTP(w, req)
    return w
}

// validOrderBody returns a valid create order request body
func validOrderBody() map[string]interface{} {
    return map[string]interface{}{
        "items": []map[string]interface{}{
            {"product_id": "le-001", "quantity": 1},
        },
        "shipping_address": map[string]interface{}{
            "facility_name": "Kennedy Space Center",
            "site_code":     "LC-39A",
            "address_line_1": "Space Commerce Way",
            "city":           "Merritt Island",
            "country":        "US",
            "postal_code":    "32953",
        },
    }
}

// --- CreateOrder tests ---

func TestCreateOrder_Success(t *testing.T) {
    r, _ := setupOrderRouter()
    token := registerAndLogin(t, r, uniqueEmail(t))

    w := authRequest(r, "POST", "/api/orders", token, validOrderBody())

    assert.Equal(t, http.StatusCreated, w.Code)

    var order models.Order
    err := json.Unmarshal(w.Body.Bytes(), &order)
    require.NoError(t, err)

    assert.NotEmpty(t, order.ID)
    assert.Equal(t, models.StatusPending, order.Status)
    assert.Equal(t, 1, len(order.Items))
    assert.Greater(t, order.Total, 0.0)
    assert.Greater(t, order.Subtotal, 0.0)
    assert.Greater(t, order.ShippingCost, 0.0)
}

func TestCreateOrder_CalculatesCorrectTotal(t *testing.T) {
    r, _ := setupOrderRouter()
    token := registerAndLogin(t, r, uniqueEmail(t))

    w := authRequest(r, "POST", "/api/orders", token, map[string]interface{}{
        "items": []map[string]interface{}{
            {"product_id": "le-001", "quantity": 2},
        },
        "shipping_address": map[string]interface{}{
            "facility_name":  "Kennedy Space Center",
            "site_code":      "LC-39A",
            "address_line_1": "Space Commerce Way",
            "city":           "Merritt Island",
            "country":        "US",
            "postal_code":    "32953",
        },
    })

    assert.Equal(t, http.StatusCreated, w.Code)

    var order models.Order
    err := json.Unmarshal(w.Body.Bytes(), &order)
    require.NoError(t, err)

    // le-001 is $4200 * 2 = $8400 subtotal
    assert.Equal(t, 8400000.0, order.Subtotal)
    assert.Equal(t, order.Subtotal+order.ShippingCost, order.Total)
}

func TestCreateOrder_SetsUserID(t *testing.T) {
    r, _ := setupOrderRouter()
    token := registerAndLogin(t, r, uniqueEmail(t))

    w := authRequest(r, "POST", "/api/orders", token, validOrderBody())

    var order models.Order
    err := json.Unmarshal(w.Body.Bytes(), &order)
    require.NoError(t, err)

    assert.NotEmpty(t, order.UserID)
}

func TestCreateOrder_InvalidProductID_Returns422(t *testing.T) {
    r, _ := setupOrderRouter()
    token := registerAndLogin(t, r, uniqueEmail(t))

    w := authRequest(r, "POST", "/api/orders", token, map[string]interface{}{
        "items": []map[string]interface{}{
            {"product_id": "does-not-exist", "quantity": 1},
        },
        "shipping_address": map[string]interface{}{
            "facility_name":  "Kennedy Space Center",
            "site_code":      "LC-39A",
            "address_line_1": "Space Commerce Way",
            "city":           "Merritt Island",
            "country":        "US",
            "postal_code":    "32953",
        },
    })

    assert.Equal(t, http.StatusUnprocessableEntity, w.Code)
}

func TestCreateOrder_MissingItems_Returns400(t *testing.T) {
    r, _ := setupOrderRouter()
    token := registerAndLogin(t, r, uniqueEmail(t))

    w := authRequest(r, "POST", "/api/orders", token, map[string]interface{}{
        "shipping_address": map[string]interface{}{
            "facility_name":  "Kennedy Space Center",
            "site_code":      "LC-39A",
            "address_line_1": "Space Commerce Way",
            "city":           "Merritt Island",
            "country":        "US",
            "postal_code":    "32953",
        },
    })

    assert.Equal(t, http.StatusBadRequest, w.Code)
}

func TestCreateOrder_MissingShippingAddress_Returns400(t *testing.T) {
    r, _ := setupOrderRouter()
    token := registerAndLogin(t, r, uniqueEmail(t))

    w := authRequest(r, "POST", "/api/orders", token, map[string]interface{}{
        "items": []map[string]interface{}{
            {"product_id": "le-001", "quantity": 1},
        },
    })

    assert.Equal(t, http.StatusBadRequest, w.Code)
}

func TestCreateOrder_NoAuth_Returns401(t *testing.T) {
    r, _ := setupOrderRouter()

    b, _ := json.Marshal(validOrderBody())
    req, _ := http.NewRequest("POST", "/api/orders", bytes.NewBuffer(b))
    req.Header.Set("Content-Type", "application/json")
    w := httptest.NewRecorder()
    r.ServeHTTP(w, req)

    assert.Equal(t, http.StatusUnauthorized, w.Code)
}

// --- GetOrders tests ---

func TestGetOrders_ReturnsEmptyForNewUser(t *testing.T) {
    r, _ := setupOrderRouter()
    token := registerAndLogin(t, r, uniqueEmail(t))

    w := authRequest(r, "GET", "/api/orders", token, nil)

    assert.Equal(t, http.StatusOK, w.Code)

    var resp models.OrderListResponse
    err := json.Unmarshal(w.Body.Bytes(), &resp)
    require.NoError(t, err)

    assert.Equal(t, 0, resp.Total)
    assert.Empty(t, resp.Data)
}

func TestGetOrders_ReturnsUserOrders(t *testing.T) {
    r, _ := setupOrderRouter()
    token := registerAndLogin(t, r, uniqueEmail(t))

    // Create two orders
    authRequest(r, "POST", "/api/orders", token, validOrderBody())
    authRequest(r, "POST", "/api/orders", token, validOrderBody())

    w := authRequest(r, "GET", "/api/orders", token, nil)

    assert.Equal(t, http.StatusOK, w.Code)

    var resp models.OrderListResponse
    err := json.Unmarshal(w.Body.Bytes(), &resp)
    require.NoError(t, err)

    assert.Equal(t, 2, resp.Total)
}

func TestGetOrders_OnlyReturnsOwnOrders(t *testing.T) {
    r, _ := setupOrderRouter()

    token1 := registerAndLogin(t, r, "user1_"+uniqueEmail(t))
    token2 := registerAndLogin(t, r, "user2_"+uniqueEmail(t))

    // User 1 creates 2 orders, user 2 creates 1
    authRequest(r, "POST", "/api/orders", token1, validOrderBody())
    authRequest(r, "POST", "/api/orders", token1, validOrderBody())
    authRequest(r, "POST", "/api/orders", token2, validOrderBody())

    w := authRequest(r, "GET", "/api/orders", token1, nil)

    var resp models.OrderListResponse
    err := json.Unmarshal(w.Body.Bytes(), &resp)
    require.NoError(t, err)

    assert.Equal(t, 2, resp.Total)
}

func TestGetOrders_NoAuth_Returns401(t *testing.T) {
    r, _ := setupOrderRouter()
    req, _ := http.NewRequest("GET", "/api/orders", nil)
    w := httptest.NewRecorder()
    r.ServeHTTP(w, req)
    assert.Equal(t, http.StatusUnauthorized, w.Code)
}

// --- GetOrderByID tests ---

func TestGetOrderByID_Success(t *testing.T) {
    r, _ := setupOrderRouter()
    token := registerAndLogin(t, r, uniqueEmail(t))

    // Create order and get its ID
    wCreate := authRequest(r, "POST", "/api/orders", token, validOrderBody())
    var created models.Order
    err := json.Unmarshal(wCreate.Body.Bytes(), &created)
    require.NoError(t, err)

    // Fetch by ID
    w := authRequest(r, "GET", "/api/orders/"+created.ID, token, nil)

    assert.Equal(t, http.StatusOK, w.Code)

    var order models.Order
    err = json.Unmarshal(w.Body.Bytes(), &order)
    require.NoError(t, err)

    assert.Equal(t, created.ID, order.ID)
}

func TestGetOrderByID_NotFound_Returns404(t *testing.T) {
    r, _ := setupOrderRouter()
    token := registerAndLogin(t, r, uniqueEmail(t))

    w := authRequest(r, "GET", "/api/orders/does-not-exist", token, nil)
    assert.Equal(t, http.StatusNotFound, w.Code)
}

func TestGetOrderByID_OtherUsersOrder_Returns403(t *testing.T) {
    r, _ := setupOrderRouter()

    token1 := registerAndLogin(t, r, "owner_"+uniqueEmail(t))
    token2 := registerAndLogin(t, r, "other_"+uniqueEmail(t))

    // User 1 creates an order
    wCreate := authRequest(r, "POST", "/api/orders", token1, validOrderBody())
    var created models.Order
    err := json.Unmarshal(wCreate.Body.Bytes(), &created)
    require.NoError(t, err)

    // User 2 tries to access it
    w := authRequest(r, "GET", "/api/orders/"+created.ID, token2, nil)
    assert.Equal(t, http.StatusForbidden, w.Code)
}

// --- CancelOrder tests ---

func TestCancelOrder_Success(t *testing.T) {
    r, _ := setupOrderRouter()
    token := registerAndLogin(t, r, uniqueEmail(t))

    wCreate := authRequest(r, "POST", "/api/orders", token, validOrderBody())
    var created models.Order
    err := json.Unmarshal(wCreate.Body.Bytes(), &created)
    require.NoError(t, err)

    w := authRequest(r, "PUT", "/api/orders/"+created.ID+"/cancel", token, nil)

    assert.Equal(t, http.StatusOK, w.Code)

    var order models.Order
    err = json.Unmarshal(w.Body.Bytes(), &order)
    require.NoError(t, err)

    assert.Equal(t, models.StatusCancelled, order.Status)
}

func TestCancelOrder_AlreadyCancelled_Returns400(t *testing.T) {
    r, _ := setupOrderRouter()
    token := registerAndLogin(t, r, uniqueEmail(t))

    wCreate := authRequest(r, "POST", "/api/orders", token, validOrderBody())
    var created models.Order
    json.Unmarshal(wCreate.Body.Bytes(), &created)

    // Cancel once
    authRequest(r, "PUT", "/api/orders/"+created.ID+"/cancel", token, nil)

    // Cancel again — should fail
    w := authRequest(r, "PUT", "/api/orders/"+created.ID+"/cancel", token, nil)
    assert.Equal(t, http.StatusBadRequest, w.Code)
}

func TestCancelOrder_NotFound_Returns404(t *testing.T) {
    r, _ := setupOrderRouter()
    token := registerAndLogin(t, r, uniqueEmail(t))

    w := authRequest(r, "PUT", "/api/orders/does-not-exist/cancel", token, nil)
    assert.Equal(t, http.StatusNotFound, w.Code)
}

func TestCancelOrder_OtherUsersOrder_Returns403(t *testing.T) {
    r, _ := setupOrderRouter()

    token1 := registerAndLogin(t, r, "owner_"+uniqueEmail(t))
    token2 := registerAndLogin(t, r, "other_"+uniqueEmail(t))

    wCreate := authRequest(r, "POST", "/api/orders", token1, validOrderBody())
    var created models.Order
    json.Unmarshal(wCreate.Body.Bytes(), &created)

    w := authRequest(r, "PUT", "/api/orders/"+created.ID+"/cancel", token2, nil)
    assert.Equal(t, http.StatusForbidden, w.Code)
}

func TestCancelOrder_NoAuth_Returns401(t *testing.T) {
    r, _ := setupOrderRouter()
    req, _ := http.NewRequest("PUT", "/api/orders/some-id/cancel", nil)
    w := httptest.NewRecorder()
    r.ServeHTTP(w, req)
    assert.Equal(t, http.StatusUnauthorized, w.Code)
}