package main

import (
	"log"
	"os"

	"github.com/BGHProjects/starbound/gateway/internal/db"
	"github.com/BGHProjects/starbound/gateway/internal/handlers"
	"github.com/BGHProjects/starbound/gateway/internal/middleware"
	"github.com/gin-gonic/gin"
	"github.com/joho/godotenv"
)

func main() {
    if err := godotenv.Load(); err != nil {
        log.Println("No .env file — reading from environment")
    }

    // Initialise database
    database, err := db.NewDB()
    if err != nil {
        log.Fatalf("Failed to initialise database: %v", err)
    }

    // Initialise user store
    userStore, err := db.NewUserStore()
    if err != nil {
        log.Fatalf("Failed to initialise user store: %v", err)
    }

    // Initialise handlers
    productHandler := handlers.NewProductHandler(database)
    authHandler    := handlers.NewAuthHandler(userStore)

    // Router
    r := gin.Default()

    // CORS
    r.Use(func(c *gin.Context) {
        c.Header("Access-Control-Allow-Origin",  "*")
        c.Header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        c.Header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        if c.Request.Method == "OPTIONS" {
            c.AbortWithStatus(204)
            return
        }
        c.Next()
    })

    // Health check
    r.GET("/health", func(c *gin.Context) {
        c.JSON(200, gin.H{
            "status":  "ok",
            "service": "starbound-gateway",
        })
    })

    // API routes
    api := r.Group("/api")
    {
        // Auth — public routes
        auth := api.Group("/auth")
        {
            auth.POST("/register", authHandler.Register)
            auth.POST("/login",    authHandler.Login)
            auth.POST("/logout",   authHandler.Logout)
            auth.GET("/me",        middleware.RequireAuth(), authHandler.Me)
        }

        // Products — public routes
        products := api.Group("/products")
        {
            products.GET("",        productHandler.GetProducts)
            products.GET("/groups", productHandler.GetProductGroups)
            products.GET("/:id",    productHandler.GetProductByID)
        }
    }

    port := os.Getenv("GATEWAY_PORT")
    if port == "" {
        port = "8000"
    }

    log.Printf("Starbound gateway running on :%s", port)
    if err := r.Run(":" + port); err != nil {
        log.Fatal(err)
    }
}