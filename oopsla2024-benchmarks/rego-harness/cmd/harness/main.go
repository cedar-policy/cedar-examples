package main

import (
	"rego-harness/harness"
)


func main() {
    inputs := harness.ReadInput()
    outputs := runExperiment(inputs)
    harness.WriteOutput(outputs)
}

func runExperiment(input harness.Input) []harness.TestData {
    outputs := make([]harness.TestData, len(input.Requests), len(input.Requests))
    policyObj := harness.ReadPolicy(input.Policy, input.Namespace)

    for i, request := range input.Requests {
        casted := request.(map[string]interface{})
        output := harness.IsAuthorized(casted, policyObj)
        outputs[i] = output
    }

    return outputs
}
