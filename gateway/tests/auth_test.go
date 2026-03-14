package tests

import (
	"bytes"
	"encoding/json"
	"fmt"
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

// setupAuthRouter creates a test router with auth routes registered
func setupAuthRouter() *gin.Engine {
    gin.SetMode(gin.TestMode)

    userStore, err := db.NewTestUserStore()
    if err != nil {
        panic("failed to initialise test user store: " + err.Error())
    }

    authHandler := handlers.NewAuthHandler(userStore)

    r := gin.New()
    auth := r.Group("/api/auth")
    {
        auth.POST("/register", authHandler.Register)
        auth.POST("/login",    authHandler.Login)
        auth.POST("/logout",   authHandler.Logout)
        auth.GET("/me",        middleware.RequireAuth(), authHandler.Me)
    }

    return r
}

// uniqueEmail generates a unique email for each test run
// so tests don't conflict with each other in the JSON store
func uniqueEmail(t *testing.T) string {
    return fmt.Sprintf("test_%s@starbound.com", t.Name())
}

func postJSON(r *gin.Engine, path string, body interface{}) *httptest.ResponseRecorder {
    b, _ := json.Marshal(body)
    w := httptest.NewRecorder()
    req, _ := http.NewRequest("POST", path, bytes.NewBuffer(b))
    req.Header.Set("Content-Type", "application/json")
    r.ServeHTTP(w, req)
    return w
}

// --- Register tests ---

func TestRegister_Success(t *testing.T) {
    r := setupAuthRouter()
    w := postJSON(r, "/api/auth/register", map[string]string{
        "email":    uniqueEmail(t),
        "name":     "Test User",
        "password": "password123",
    })

    assert.Equal(t, http.StatusCreated, w.Code)

    var resp models.AuthResponse
    err := json.Unmarshal(w.Body.Bytes(), &resp)
    require.NoError(t, err)

    assert.NotEmpty(t, resp.Token)
    assert.NotEmpty(t, resp.User.ID)
    assert.NotEmpty(t, resp.User.CreatedAt)
}

func TestRegister_ReturnsUserWithoutPassword(t *testing.T) {
    r := setupAuthRouter()
    w := postJSON(r, "/api/auth/register", map[string]string{
        "email":    uniqueEmail(t),
        "name":     "Test User",
        "password": "password123",
    })

    assert.Equal(t, http.StatusCreated, w.Code)

    // Raw check — hashed_password must never appear in response
    assert.NotContains(t, w.Body.String(), "hashed_password")
    assert.NotContains(t, w.Body.String(), "password")
}

func TestRegister_DuplicateEmail_Returns409(t *testing.T) {
    r := setupAuthRouter()
    email := uniqueEmail(t)

    body := map[string]string{
        "email":    email,
        "name":     "Test User",
        "password": "password123",
    }

    // First registration should succeed
    w1 := postJSON(r, "/api/auth/register", body)
    assert.Equal(t, http.StatusCreated, w1.Code)

    // Second registration with same email should fail
    w2 := postJSON(r, "/api/auth/register", body)
    assert.Equal(t, http.StatusConflict, w2.Code)
}

func TestRegister_MissingEmail_Returns400(t *testing.T) {
    r := setupAuthRouter()
    w := postJSON(r, "/api/auth/register", map[string]string{
        "name":     "Test User",
        "password": "password123",
    })
    assert.Equal(t, http.StatusBadRequest, w.Code)
}

func TestRegister_InvalidEmail_Returns400(t *testing.T) {
    r := setupAuthRouter()
    w := postJSON(r, "/api/auth/register", map[string]string{
        "email":    "notanemail",
        "name":     "Test User",
        "password": "password123",
    })
    assert.Equal(t, http.StatusBadRequest, w.Code)
}

func TestRegister_ShortPassword_Returns400(t *testing.T) {
    r := setupAuthRouter()
    w := postJSON(r, "/api/auth/register", map[string]string{
        "email":    uniqueEmail(t),
        "name":     "Test User",
        "password": "short",
    })
    assert.Equal(t, http.StatusBadRequest, w.Code)
}

func TestRegister_ShortName_Returns400(t *testing.T) {
    r := setupAuthRouter()
    w := postJSON(r, "/api/auth/register", map[string]string{
        "email":    uniqueEmail(t),
        "name":     "A",
        "password": "password123",
    })
    assert.Equal(t, http.StatusBadRequest, w.Code)
}

// --- Login tests ---

func TestLogin_Success(t *testing.T) {
    r := setupAuthRouter()
    email := uniqueEmail(t)

    // Register first
    postJSON(r, "/api/auth/register", map[string]string{
        "email":    email,
        "name":     "Test User",
        "password": "password123",
    })

    // Then login
    w := postJSON(r, "/api/auth/login", map[string]string{
        "email":    email,
        "password": "password123",
    })

    assert.Equal(t, http.StatusOK, w.Code)

    var resp models.AuthResponse
    err := json.Unmarshal(w.Body.Bytes(), &resp)
    require.NoError(t, err)

    assert.NotEmpty(t, resp.Token)
    assert.Equal(t, email, resp.User.Email)
}

func TestLogin_WrongPassword_Returns401(t *testing.T) {
    r := setupAuthRouter()
    email := uniqueEmail(t)

    postJSON(r, "/api/auth/register", map[string]string{
        "email":    email,
        "name":     "Test User",
        "password": "password123",
    })

    w := postJSON(r, "/api/auth/login", map[string]string{
        "email":    email,
        "password": "wrongpassword",
    })

    assert.Equal(t, http.StatusUnauthorized, w.Code)
}

func TestLogin_UnknownEmail_Returns401(t *testing.T) {
    r := setupAuthRouter()
    w := postJSON(r, "/api/auth/login", map[string]string{
        "email":    "nobody@starbound.com",
        "password": "password123",
    })
    assert.Equal(t, http.StatusUnauthorized, w.Code)
}

func TestLogin_MissingFields_Returns400(t *testing.T) {
    r := setupAuthRouter()
    w := postJSON(r, "/api/auth/login", map[string]string{
        "email": "test@starbound.com",
    })
    assert.Equal(t, http.StatusBadRequest, w.Code)
}

// --- Logout tests ---

func TestLogout_Returns200(t *testing.T) {
    r := setupAuthRouter()
    w := postJSON(r, "/api/auth/logout", nil)
    assert.Equal(t, http.StatusOK, w.Code)
}

// --- Me tests ---

func TestMe_WithValidToken_ReturnsUser(t *testing.T) {
    r := setupAuthRouter()
    email := uniqueEmail(t)

    // Register and grab token
    w := postJSON(r, "/api/auth/register", map[string]string{
        "email":    email,
        "name":     "Test User",
        "password": "password123",
    })

    var resp models.AuthResponse
    err := json.Unmarshal(w.Body.Bytes(), &resp)
    require.NoError(t, err)

    // Call /me with token
    req, _ := http.NewRequest("GET", "/api/auth/me", nil)
    req.Header.Set("Authorization", "Bearer "+resp.Token)
    wMe := httptest.NewRecorder()
    r.ServeHTTP(wMe, req)

    assert.Equal(t, http.StatusOK, wMe.Code)

    var user models.UserPublic
    err = json.Unmarshal(wMe.Body.Bytes(), &user)
    require.NoError(t, err)

    assert.Equal(t, email, user.Email)
    assert.NotEmpty(t, user.ID)
}

func TestMe_WithNoToken_Returns401(t *testing.T) {
    r := setupAuthRouter()
    req, _ := http.NewRequest("GET", "/api/auth/me", nil)
    w := httptest.NewRecorder()
    r.ServeHTTP(w, req)
    assert.Equal(t, http.StatusUnauthorized, w.Code)
}

func TestMe_WithInvalidToken_Returns401(t *testing.T) {
    r := setupAuthRouter()
    req, _ := http.NewRequest("GET", "/api/auth/me", nil)
    req.Header.Set("Authorization", "Bearer this.is.not.valid")
    w := httptest.NewRecorder()
    r.ServeHTTP(w, req)
    assert.Equal(t, http.StatusUnauthorized, w.Code)
}

func TestMe_WithMalformedHeader_Returns401(t *testing.T) {
    r := setupAuthRouter()
    req, _ := http.NewRequest("GET", "/api/auth/me", nil)
    req.Header.Set("Authorization", "NotBearer sometoken")
    w := httptest.NewRecorder()
    r.ServeHTTP(w, req)
    assert.Equal(t, http.StatusUnauthorized, w.Code)
}