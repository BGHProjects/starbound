package main

import (
	"log"
	"os"

	"github.com/gin-gonic/gin"
	"github.com/joho/godotenv"
	swaggerFiles "github.com/swaggo/files"
	ginSwagger "github.com/swaggo/gin-swagger"

	_ "github.com/BGHProjects/starbound/gateway/docs"
	"github.com/BGHProjects/starbound/gateway/internal/db"
	"github.com/BGHProjects/starbound/gateway/internal/handlers"
	"github.com/BGHProjects/starbound/gateway/internal/middleware"
)

// @title           Starbound API
// @version         1.0
// @description     API gateway for the Starbound rocket parts e-commerce platform
// @host            localhost:8000
// @BasePath        /api
// @securityDefinitions.apikey BearerAuth
// @in              header
// @name            Authorization
// @description     JWT token — format: Bearer <token>
func main() {
	if err := godotenv.Load(); err != nil {
		log.Println("No .env file — reading from environment")
	}

	log.Println("Initialising product database...")
	database, err := db.NewDB()
	if err != nil {
		log.Fatalf("Failed to initialise database: %v", err)
	}
	log.Println("Product database OK")

	log.Println("Initialising user store...")
	userStore, err := db.NewUserStore()
	if err != nil {
		log.Fatalf("Failed to initialise user store: %v", err)
	}
	log.Println("User store OK")

	log.Println("Initialising order store...")
	orderStore, err := db.NewOrderStore()
	if err != nil {
		log.Fatalf("Failed to initialise order store: %v", err)
	}
	log.Println("Order store OK")

	productHandler := handlers.NewProductHandler(database)
	authHandler    := handlers.NewAuthHandler(userStore)
	orderHandler   := handlers.NewOrderHandler(orderStore, database)

	r := gin.Default()

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

	r.GET("/swagger/*any", ginSwagger.WrapHandler(swaggerFiles.Handler))

	r.GET("/health", func(c *gin.Context) {
		c.JSON(200, gin.H{
			"status":  "ok",
			"service": "starbound-gateway",
		})
	})

	api := r.Group("/api")
	{
		auth := api.Group("/auth")
		{
			auth.POST("/register", authHandler.Register)
			auth.POST("/login",    authHandler.Login)
			auth.POST("/logout",   authHandler.Logout)
			auth.GET("/me",        middleware.RequireAuth(), authHandler.Me)
		}

		products := api.Group("/products")
		{
			products.GET("",        productHandler.GetProducts)
			products.GET("/groups", productHandler.GetProductGroups)
			products.GET("/:id",    productHandler.GetProductByID)
		}

		orders := api.Group("/orders", middleware.RequireAuth())
		{
			orders.GET("",            orderHandler.GetOrders)
			orders.POST("",           orderHandler.CreateOrder)
			orders.GET("/:id",        orderHandler.GetOrderByID)
			orders.PUT("/:id/cancel", orderHandler.CancelOrder)
		}

		api.POST("/chat",             handlers.ProxyChat)
		api.POST("/refund/validate",  handlers.ProxyRefund)
	}

	port := os.Getenv("GATEWAY_PORT")
	if port == "" {
		port = "8000"
	}

	log.Printf("Starbound gateway running on :%s", port)
	log.Printf("Swagger UI at http://localhost:%s/swagger/index.html", port)
	if err := r.Run(":" + port); err != nil {
		log.Fatal(err)
	}
}