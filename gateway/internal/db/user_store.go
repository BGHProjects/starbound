package db

import (
	"encoding/json"
	"fmt"
	"os"
	"sync"
	"time"

	"github.com/BGHProjects/starbound/gateway/internal/models"
	"github.com/google/uuid"
	"golang.org/x/crypto/bcrypt"
)

// UserStore handles reading and writing users to a local JSON file
type UserStore struct {
    mu       sync.RWMutex
    filePath string
    users    []models.User
}

// NewUserStore creates a UserStore backed by the given JSON file
func NewUserStore() (*UserStore, error) {
    return newUserStore("internal/db/users.json")
}

func newUserStore(path string) (*UserStore, error) {
    store := &UserStore{filePath: path}
    if err := store.load(); err != nil {
        return nil, err
    }
    return store, nil
}

// load reads users from the JSON file into memory
func (s *UserStore) load() error {
    data, err := os.ReadFile(s.filePath)
    if err != nil {
        return fmt.Errorf("failed to read users file: %w", err)
    }
    return json.Unmarshal(data, &s.users)
}

// save writes the current in-memory users back to the JSON file
func (s *UserStore) save() error {
    data, err := json.MarshalIndent(s.users, "", "  ")
    if err != nil {
        return fmt.Errorf("failed to marshal users: %w", err)
    }
    return os.WriteFile(s.filePath, data, 0644)
}

// FindByEmail returns a user by email, or nil if not found
func (s *UserStore) FindByEmail(email string) *models.User {
    s.mu.RLock()
    defer s.mu.RUnlock()
    for i, u := range s.users {
        if u.Email == email {
            return &s.users[i]
        }
    }
    return nil
}

// FindByID returns a user by ID, or nil if not found
func (s *UserStore) FindByID(id string) *models.User {
    s.mu.RLock()
    defer s.mu.RUnlock()
    for i, u := range s.users {
        if u.ID == id {
            return &s.users[i]
        }
    }
    return nil
}

// CreateUser hashes the password and saves a new user
func (s *UserStore) CreateUser(req models.RegisterRequest) (*models.User, error) {
    s.mu.Lock()
    defer s.mu.Unlock()

    // Check email not already taken
    for _, u := range s.users {
        if u.Email == req.Email {
            return nil, fmt.Errorf("email already registered")
        }
    }

    hashed, err := bcrypt.GenerateFromPassword([]byte(req.Password), bcrypt.DefaultCost)
    if err != nil {
        return nil, fmt.Errorf("failed to hash password: %w", err)
    }

    user := models.User{
        ID:             uuid.New().String(),
        Email:          req.Email,
        Name:           req.Name,
        HashedPassword: string(hashed),
        CreatedAt:      time.Now().UTC().Format(time.RFC3339),
    }

    s.users = append(s.users, user)

    if err := s.save(); err != nil {
        return nil, fmt.Errorf("failed to save user: %w", err)
    }

    return &user, nil
}

// CheckPassword verifies a plain password against a hashed one
func CheckPassword(plain, hashed string) bool {
    err := bcrypt.CompareHashAndPassword([]byte(hashed), []byte(plain))
    return err == nil
}