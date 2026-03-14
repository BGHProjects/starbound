package handlers

import (
	"net/http"

	"github.com/BGHProjects/starbound/gateway/internal/db"
	"github.com/BGHProjects/starbound/gateway/internal/middleware"
	"github.com/BGHProjects/starbound/gateway/internal/models"
	"github.com/gin-gonic/gin"
)

type AuthHandler struct {
    users *db.UserStore
}

func NewAuthHandler(users *db.UserStore) *AuthHandler {
    return &AuthHandler{users: users}
}

// Register handles POST /api/auth/register
func (h *AuthHandler) Register(c *gin.Context) {
    var req models.RegisterRequest
    if err := c.ShouldBindJSON(&req); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{
            "error": err.Error(),
        })
        return
    }

    user, err := h.users.CreateUser(req)
    if err != nil {
        if err.Error() == "email already registered" {
            c.JSON(http.StatusConflict, gin.H{
                "error": "an account with that email already exists",
            })
            return
        }
        c.JSON(http.StatusInternalServerError, gin.H{
            "error": "failed to create account",
        })
        return
    }

    token, err := middleware.GenerateToken(user.ID, user.Email, user.Name)
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{
            "error": "failed to generate token",
        })
        return
    }

    c.JSON(http.StatusCreated, models.AuthResponse{
        Token: token,
        User: models.UserPublic{
            ID:        user.ID,
            Email:     user.Email,
            Name:      user.Name,
            CreatedAt: user.CreatedAt,
        },
    })
}

// Login handles POST /api/auth/login
func (h *AuthHandler) Login(c *gin.Context) {
    var req models.LoginRequest
    if err := c.ShouldBindJSON(&req); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{
            "error": err.Error(),
        })
        return
    }

    user := h.users.FindByEmail(req.Email)
    if user == nil || !db.CheckPassword(req.Password, user.HashedPassword) {
        // Same error for both cases — don't reveal whether email exists
        c.JSON(http.StatusUnauthorized, gin.H{
            "error": "invalid email or password",
        })
        return
    }

    token, err := middleware.GenerateToken(user.ID, user.Email, user.Name)
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{
            "error": "failed to generate token",
        })
        return
    }

    c.JSON(http.StatusOK, models.AuthResponse{
        Token: token,
        User: models.UserPublic{
            ID:        user.ID,
            Email:     user.Email,
            Name:      user.Name,
            CreatedAt: user.CreatedAt,
        },
    })
}

// Logout handles POST /api/auth/logout
// JWT is stateless so logout is handled client-side by discarding the token.
// This endpoint exists so the frontend has a consistent API to call.
func (h *AuthHandler) Logout(c *gin.Context) {
    c.JSON(http.StatusOK, gin.H{
        "message": "logged out successfully",
    })
}

// Me handles GET /api/auth/me
// Returns the current user from the JWT claims set by RequireAuth middleware
func (h *AuthHandler) Me(c *gin.Context) {
    userID := c.GetString("user_id")

    user := h.users.FindByID(userID)
    if user == nil {
        c.JSON(http.StatusNotFound, gin.H{
            "error": "user not found",
        })
        return
    }

    c.JSON(http.StatusOK, models.UserPublic{
        ID:        user.ID,
        Email:     user.Email,
        Name:      user.Name,
        CreatedAt: user.CreatedAt,
    })
}