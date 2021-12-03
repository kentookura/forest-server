package main

import (
	"fmt"

	"github.com/gin-gonic/gin"
)

func setupRouter() *gin.Engine {
	r := gin.Default()

	r.GET("/hello", func(c *gin.Context) {
		what := c.Query("what")
		if what == "" {
			what = "World"
		}

		c.String(200, fmt.Sprintf("Hello, %s!", what))
	})

	return r
}

func main() {
	r := setupRouter()

	r.Run(":3000")
}
