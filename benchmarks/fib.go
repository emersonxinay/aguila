package main

import (
	"fmt"
	"time"
)

func fib(n int) int {
	if n <= 1 {
		return n
	}
	return fib(n-1) + fib(n-2)
}

func main() {
	start := time.Now()
	res := fib(40)
	elapsed := time.Since(start)
	fmt.Printf("Go: Fib(40) = %d\n", res)
	fmt.Printf("Tiempo: %f segundos\n", elapsed.Seconds())
}
