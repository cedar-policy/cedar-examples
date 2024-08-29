// Package entitytype contains the enum EntityType that represents the different entity types supported by
// entitystore.EntityStore.
package entitytype

import (
	"strings"
)

// EntityType is an enum that represents the different entity types supported by Cedar.
type EntityType int

const (
	Unknown EntityType = iota
	User
	Team
	Application
	Action
	List
	Task
)

var (
	Name = map[EntityType]string{
		Unknown:     "unknown",
		User:        "User",
		Team:        "Team",
		Application: "Application",
		Action:      "Action",
		List:        "List",
		Task:        "Task",
	}
)

func (e EntityType) String() string {
	return Name[e]
}

// Parse parses a given string as EntityType using strings.EqualFold() (case insensitive)
func Parse(et string) EntityType {
	for res := range Name {
		if strings.EqualFold(res.String(), et) {
			return res
		}
	}
	return Unknown
}
