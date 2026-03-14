package tests

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/BGHProjects/starbound/gateway/internal/db"
	"github.com/BGHProjects/starbound/gateway/internal/handlers"
	"github.com/BGHProjects/starbound/gateway/internal/models"
	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func setupRouter() *gin.Engine {
    gin.SetMode(gin.TestMode)

    database, err := db.NewTestDB()
    if err != nil {
        panic("failed to initialise test database: " + err.Error())
    }

    productHandler := handlers.NewProductHandler(database)

    r := gin.New()
    api := r.Group("/api")
    {
        products := api.Group("/products")
        {
            products.GET("",        productHandler.GetProducts)
            products.GET("/groups", productHandler.GetProductGroups)
            products.GET("/:id",    productHandler.GetProductByID)
        }
    }

    return r
}

func TestGetProducts_ReturnsOK(t *testing.T) {
    r := setupRouter()
    w := httptest.NewRecorder()
    req, _ := http.NewRequest("GET", "/api/products", nil)
    r.ServeHTTP(w, req)

    assert.Equal(t, http.StatusOK, w.Code)
}

func TestGetProducts_ResponseShape(t *testing.T) {
    r := setupRouter()
    w := httptest.NewRecorder()
    req, _ := http.NewRequest("GET", "/api/products", nil)
    r.ServeHTTP(w, req)

    var response models.ProductListResponse
    err := json.Unmarshal(w.Body.Bytes(), &response)
    require.NoError(t, err)

    assert.GreaterOrEqual(t, response.Total, 0)
    assert.Equal(t, 1, response.Page)
    assert.Equal(t, 20, response.Limit)
    assert.NotNil(t, response.Data)
}

func TestGetProducts_FilterByGroup(t *testing.T) {
    r := setupRouter()
    w := httptest.NewRecorder()
    req, _ := http.NewRequest("GET", "/api/products?group=propulsion", nil)
    r.ServeHTTP(w, req)

    assert.Equal(t, http.StatusOK, w.Code)

    var response models.ProductListResponse
    err := json.Unmarshal(w.Body.Bytes(), &response)
    require.NoError(t, err)

    for _, p := range response.Data {
        assert.Equal(t, models.GroupPropulsion, p.Group)
    }
}

func TestGetProducts_FilterByType(t *testing.T) {
    r := setupRouter()
    w := httptest.NewRecorder()
    req, _ := http.NewRequest("GET", "/api/products?type=liquid_engine", nil)
    r.ServeHTTP(w, req)

    assert.Equal(t, http.StatusOK, w.Code)

    var response models.ProductListResponse
    err := json.Unmarshal(w.Body.Bytes(), &response)
    require.NoError(t, err)

    for _, p := range response.Data {
        assert.Equal(t, models.TypeLiquidEngine, p.Type)
    }
}

func TestGetProducts_SearchByName(t *testing.T) {
    r := setupRouter()
    w := httptest.NewRecorder()
    req, _ := http.NewRequest("GET", "/api/products?search=merlin", nil)
    r.ServeHTTP(w, req)

    assert.Equal(t, http.StatusOK, w.Code)

    var response models.ProductListResponse
    err := json.Unmarshal(w.Body.Bytes(), &response)
    require.NoError(t, err)

    for _, p := range response.Data {
        assert.Contains(t,
            strings.ToLower(p.Name),
            "merlin",
        )
    }
}

func TestGetProducts_Pagination(t *testing.T) {
    r := setupRouter()
    w := httptest.NewRecorder()
    req, _ := http.NewRequest("GET", "/api/products?page=1&limit=2", nil)
    r.ServeHTTP(w, req)

    assert.Equal(t, http.StatusOK, w.Code)

    var response models.ProductListResponse
    err := json.Unmarshal(w.Body.Bytes(), &response)
    require.NoError(t, err)

    assert.LessOrEqual(t, len(response.Data), 2)
    assert.Equal(t, 1, response.Page)
    assert.Equal(t, 2, response.Limit)
}

func TestGetProducts_NoMatch_ReturnsEmpty(t *testing.T) {
    r := setupRouter()
    w := httptest.NewRecorder()
    req, _ := http.NewRequest("GET", "/api/products?search=zzznomatch", nil)
    r.ServeHTTP(w, req)

    assert.Equal(t, http.StatusOK, w.Code)

    var response models.ProductListResponse
    err := json.Unmarshal(w.Body.Bytes(), &response)
    require.NoError(t, err)

    assert.Equal(t, 0, response.Total)
    assert.Empty(t, response.Data)
}

func TestGetProductByID_ValidID_ReturnsProduct(t *testing.T) {
    r := setupRouter()
    w := httptest.NewRecorder()
    req, _ := http.NewRequest("GET", "/api/products/le-001", nil)
    r.ServeHTTP(w, req)

    assert.Equal(t, http.StatusOK, w.Code)

    var product models.Product
    err := json.Unmarshal(w.Body.Bytes(), &product)
    require.NoError(t, err)

    assert.Equal(t, "le-001", product.ID)
    assert.NotEmpty(t, product.Name)
    assert.NotEmpty(t, product.Attributes)
}

func TestGetProductByID_InvalidID_Returns404(t *testing.T) {
    r := setupRouter()
    w := httptest.NewRecorder()
    req, _ := http.NewRequest("GET", "/api/products/does-not-exist", nil)
    r.ServeHTTP(w, req)

    assert.Equal(t, http.StatusNotFound, w.Code)
}

func TestGetProductByID_HasFullAttributes(t *testing.T) {
    r := setupRouter()
    w := httptest.NewRecorder()
    req, _ := http.NewRequest("GET", "/api/products/le-001", nil)
    r.ServeHTTP(w, req)

    var product models.Product
    err := json.Unmarshal(w.Body.Bytes(), &product)
    require.NoError(t, err)

    assert.NotEqual(t, "null", string(product.Attributes))

    var attrs models.LiquidEngineAttributes
    err = json.Unmarshal(product.Attributes, &attrs)
    require.NoError(t, err)
    assert.Greater(t, attrs.MaxThrust, 0.0)
}

func TestGetProductGroups_ReturnsOK(t *testing.T) {
    r := setupRouter()
    w := httptest.NewRecorder()
    req, _ := http.NewRequest("GET", "/api/products/groups", nil)
    r.ServeHTTP(w, req)

    assert.Equal(t, http.StatusOK, w.Code)
}

func TestGetProductGroups_ReturnsFourGroups(t *testing.T) {
    r := setupRouter()
    w := httptest.NewRecorder()
    req, _ := http.NewRequest("GET", "/api/products/groups", nil)
    r.ServeHTTP(w, req)

    var response map[string]interface{}
    err := json.Unmarshal(w.Body.Bytes(), &response)
    require.NoError(t, err)

    groups, ok := response["data"].([]interface{})
    require.True(t, ok)
    assert.Equal(t, 4, len(groups))
}