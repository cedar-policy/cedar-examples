# Cedar Go Hello World
This repository contains a simple hello world program demonstrating typical usage of the [Cedar Go APIs](https://github.com/cedar-policy/cedar-go).  
It is designed for demonstration purposes and does not reflect a production-ready application.  

The file `main.go` provides example code, showcasing the primary steps an application takes to use Cedar.
- Parsing policies from JSON and Cedar
- Creating policies programmatically with the AST package
- Creating entity sets 
- Creating authorization requests
- Outputting and interpreting the decision from the request

## Setup and run
The example is built on Go 1.25.6 and uses cedar-go 1.6.0 - you will need to have a compatible version of Go installed.  
To run the program, navigate to the `cedar-go-hello-world` directory and run the following command:  
`go run .`