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

// Register godoc
// @Summary      Register a new account
// @Description  Creates a new user account and returns a JWT token
// @Tags         auth
// @Accept       json
// @Produce      json
// @Param        body  body      models.RegisterRequest  true  "Registration details"
// @Success      201   {object}  models.AuthResponse
// @Failure      400   {object}  map[string]string
// @Failure      409   {object}  map[string]string
// @Failure      500   {object}  map[string]string
// @Router       /auth/register [post]
func (h *AuthHandler) Register(c *gin.Context) {
    var req models.RegisterRequest
    if err := c.ShouldBindJSON(&req); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
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

// Login godoc
// @Summary      Login
// @Description  Authenticates a user and returns a JWT token
// @Tags         auth
// @Accept       json
// @Produce      json
// @Param        body  body      models.LoginRequest  true  "Login credentials"
// @Success      200   {object}  models.AuthResponse
// @Failure      400   {object}  map[string]string
// @Failure      401   {object}  map[string]string
// @Failure      500   {object}  map[string]string
// @Router       /auth/login [post]
func (h *AuthHandler) Login(c *gin.Context) {
    var req models.LoginRequest
    if err := c.ShouldBindJSON(&req); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
        return
    }

    user := h.users.FindByEmail(req.Email)
    if user == nil || !db.CheckPassword(req.Password, user.HashedPassword) {
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

// Logout godoc
// @Summary      Logout
// @Description  Stateless logout — client should discard the token
// @Tags         auth
// @Produce      json
// @Success      200  {object}  map[string]string
// @Router       /auth/logout [post]
func (h *AuthHandler) Logout(c *gin.Context) {
    c.JSON(http.StatusOK, gin.H{"message": "logged out successfully"})
}

// Me godoc
// @Summary      Get current user
// @Description  Returns the authenticated user's profile from the JWT
// @Tags         auth
// @Produce      json
// @Security     BearerAuth
// @Success      200  {object}  models.UserPublic
// @Failure      401  {object}  map[string]string
// @Failure      404  {object}  map[string]string
// @Router       /auth/me [get]
func (h *AuthHandler) Me(c *gin.Context) {
    userID := c.GetString("user_id")

    user := h.users.FindByID(userID)
    if user == nil {
        c.JSON(http.StatusNotFound, gin.H{"error": "user not found"})
        return
    }

    c.JSON(http.StatusOK, models.UserPublic{
        ID:        user.ID,
        Email:     user.Email,
        Name:      user.Name,
        CreatedAt: user.CreatedAt,
    })
}