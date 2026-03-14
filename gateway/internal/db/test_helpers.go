package db

import (
	"os"
)

// NewTestUserStore creates a UserStore backed by a temporary file
// so tests never pollute the real users.json
func NewTestUserStore() (*UserStore, error) {
    tmp, err := os.CreateTemp("", "starbound_test_users_*.json")
    if err != nil {
        return nil, err
    }
    // Write empty array so the file is valid JSON
    if _, err := tmp.WriteString("[]"); err != nil {
        return nil, err
    }
    tmp.Close()

    return newUserStore(tmp.Name())
}