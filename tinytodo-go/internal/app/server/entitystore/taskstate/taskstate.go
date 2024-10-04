// Package taskstate contains the enum TaskState that represents the different states supported by entitystore.Task.
package taskstate

import (
	"fmt"
	"strings"
)

// TaskState is an enum that represents the different states supported by entitystore.Task.
type TaskState int

const (
	Unknown TaskState = iota
	Checked
	Unchecked
)

var (
	Name = map[TaskState]string{
		Unknown:   "unknown",
		Checked:   "Checked",
		Unchecked: "Unchecked", // expected by client
	}
)

func (t TaskState) String() string {
	return Name[t]
}

// Parse parses a given string as TaskState using strings.EqualFold() (case insensitive)
func Parse(et string) TaskState {
	for res := range Name {
		if strings.EqualFold(res.String(), et) {
			return res
		}
	}
	return Unknown
}

// MarshalJSON converts a TaskState into its Cedar language representation.
//
// We override the default MarshalJSON because we want to marshal TaskState as a string (e.g., "Checked") instead of
// an integer (e.g., 1).
func (t TaskState) MarshalJSON() ([]byte, error) {
	return []byte(fmt.Sprintf("%q", t.String())), nil
}
