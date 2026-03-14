package middleware

import (
	"fmt"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
)

// Claims is the JWT payload
type Claims struct {
    UserID string `json:"user_id"`
    Email  string `json:"email"`
    Name   string `json:"name"`
    jwt.RegisteredClaims
}

// GenerateToken creates a signed JWT for the given user
func GenerateToken(userID, email, name string) (string, error) {
    secret := os.Getenv("JWT_SECRET")
    if secret == "" {
        secret = "dev_secret_change_in_production"
    }

    claims := Claims{
        UserID: userID,
        Email:  email,
        Name:   name,
        RegisteredClaims: jwt.RegisteredClaims{
            ExpiresAt: jwt.NewNumericDate(time.Now().Add(24 * time.Hour)),
            IssuedAt:  jwt.NewNumericDate(time.Now()),
            Issuer:    "starbound",
        },
    }

    token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
    return token.SignedString([]byte(secret))
}

// ParseToken validates a JWT string and returns the claims
func ParseToken(tokenStr string) (*Claims, error) {
    secret := os.Getenv("JWT_SECRET")
    if secret == "" {
        secret = "dev_secret_change_in_production"
    }

    token, err := jwt.ParseWithClaims(tokenStr, &Claims{}, func(t *jwt.Token) (interface{}, error) {
        if _, ok := t.Method.(*jwt.SigningMethodHMAC); !ok {
            return nil, fmt.Errorf("unexpected signing method: %v", t.Header["alg"])
        }
        return []byte(secret), nil
    })
    if err != nil {
        return nil, err
    }

    claims, ok := token.Claims.(*Claims)
    if !ok || !token.Valid {
        return nil, fmt.Errorf("invalid token")
    }

    return claims, nil
}

// RequireAuth is a Gin middleware that protects routes with JWT
func RequireAuth() gin.HandlerFunc {
    return func(c *gin.Context) {
        authHeader := c.GetHeader("Authorization")
        if authHeader == "" {
            c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
                "error": "authorization header required",
            })
            return
        }

        parts := strings.SplitN(authHeader, " ", 2)
        if len(parts) != 2 || strings.ToLower(parts[0]) != "bearer" {
            c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
                "error": "authorization header format must be: Bearer <token>",
            })
            return
        }

        claims, err := ParseToken(parts[1])
        if err != nil {
            c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
                "error": "invalid or expired token",
            })
            return
        }

        // Store claims in context for handlers to use
        c.Set("user_id", claims.UserID)
        c.Set("email",   claims.Email)
        c.Set("name",    claims.Name)
        c.Next()
    }
}