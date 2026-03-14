package main

import (
	"log"
	"os"

	"github.com/gin-gonic/gin"
	"github.com/joho/godotenv"
)

func main() {
    if err := godotenv.Load(); err != nil {
        log.Println("No .env file — reading from environment")
    }

    r := gin.Default()

    r.GET("/health", func(c *gin.Context) {
        c.JSON(200, gin.H{
            "status":  "ok",
            "service": "starbound-gateway",
        })
    })

    port := os.Getenv("GATEWAY_PORT")
    if port == "" {
        port = "8000"
    }

    log.Printf("Starbound gateway running on :%s", port)
    if err := r.Run(":" + port); err != nil {
        log.Fatal(err)
    }
}