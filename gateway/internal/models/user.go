package models

// User is the full user struct including hashed password
// — never returned to the client
type User struct {
    ID             string `json:"id"`
    Email          string `json:"email"`
    Name           string `json:"name"`
    HashedPassword string `json:"hashed_password"`
    CreatedAt      string `json:"created_at"`
}

// UserPublic is the safe version returned to the client
// — no password field
type UserPublic struct {
    ID        string `json:"id"`
    Email     string `json:"email"`
    Name      string `json:"name"`
    CreatedAt string `json:"created_at"`
}

// RegisterRequest is the body for POST /api/auth/register
type RegisterRequest struct {
    Email    string `json:"email"    binding:"required,email"`
    Name     string `json:"name"     binding:"required,min=2"`
    Password string `json:"password" binding:"required,min=8"`
}

// LoginRequest is the body for POST /api/auth/login
type LoginRequest struct {
    Email    string `json:"email"    binding:"required,email"`
    Password string `json:"password" binding:"required"`
}

// AuthResponse is returned on successful register or login
type AuthResponse struct {
    Token string     `json:"token"`
    User  UserPublic `json:"user"`
}