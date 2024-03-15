package main

import (
	"TinyTodoRego/tinytodo"
)


func main() {
    inputs := tinytodo.ReadInput()
    outputs := runExpirement(inputs)
    tinytodo.WriteOutput(outputs)
}

func runExpirement(input tinytodo.Input) []tinytodo.TestData {
    outputs := make([]tinytodo.TestData, len(input.Requests), len(input.Requests))
    policyObj := tinytodo.ReadPolicy(input.Policy, input.Namespace)

    for i, request := range input.Requests {
        casted := request.(map[string]interface{})
        output := tinytodo.IsAuthorized(casted, policyObj)
        outputs[i] = output
    }

    return outputs
}
