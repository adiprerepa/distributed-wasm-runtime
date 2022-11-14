package main

import "fmt"

func main() {
	fmt.Println(add(1, 2));
}

func add(x uint32, y uint32) uint32 {
	return x+y
}
