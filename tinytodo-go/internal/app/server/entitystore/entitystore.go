// Package entitystore contains the definitions for entities encountered in TinyTodo.
//
// It is translated from entitystore.rs.
package entitystore

import (
	"encoding/json"
	"fmt"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"strconv"
)

// EntityStore tracks all entities that are handled in TinyTodo.
type EntityStore struct {
	Users UserUIDToUserMap `json:"users"`
	Teams TeamUIDToTeamMap `json:"teams"`
	Lists ListUIDToListMap `json:"lists"`
	App   App              `json:"app"`
}

func New(entitiesJSON []byte) (*EntityStore, error) {
	var es EntityStore
	if err := json.Unmarshal(entitiesJSON, &es); err != nil {
		return nil, fmt.Errorf("failed to unmarshal EntityStore: %w", err)
	}
	return &es, nil
}

// GetNextListUID returns the next available ListUID (but does not create) with a monotonically increasing ID.
func (e *EntityStore) GetNextListUID() ListUID {
	var id int
	for {
		listUID := ListUID{
			EntityUID: entityuid.NewEntityUID(entitytype.List, strconv.Itoa(id)),
		}
		if _, found := e.Lists[listUID]; !found {
			return listUID
		}
		id++
	}
}

// InsertNextTeam creates the next available Team with a monotonically increasing ID and returns the TeamUID.
func (e *EntityStore) InsertNextTeam() TeamUID {
	var id int
	for {
		teamUID := TeamUID{
			EntityUID: entityuid.NewEntityUID(entitytype.Team, strconv.Itoa(id)),
		}
		if _, found := e.Teams[teamUID]; !found {
			e.Teams[teamUID] = NewTeam(teamUID, nil)
			return teamUID
		}
		id++
	}
}

// UserUIDToUserMap is a special type created to help unmarshal map[UserUID]User.
//
// Inspired by https://groups.google.com/g/golang-nuts/c/IxPipKwI-zQ and https://go.dev/play/p/YgUIFxT7hA.
type UserUIDToUserMap map[UserUID]*User

func (im *UserUIDToUserMap) UnmarshalJSON(bytes []byte) error {

	// Unmarshal the string-keyed map
	sk := make(map[string]*User)
	if err := json.Unmarshal(bytes, &sk); err != nil {
		return err
	}

	// Copy the values
	*im = make(map[UserUID]*User)
	for k, v := range sk {
		ki, err := entityuid.ParseEntityUID(k)
		if err != nil {
			return err
		}
		(*im)[UserUID{
			EntityUID: ki,
		}] = v
	}

	return nil
}

// TeamUIDToTeamMap is a special type created to help unmarshal map[TeamUID]Team.
//
// Inspired by https://groups.google.com/g/golang-nuts/c/IxPipKwI-zQ and https://go.dev/play/p/YgUIFxT7hA.
type TeamUIDToTeamMap map[TeamUID]*Team

func (im *TeamUIDToTeamMap) UnmarshalJSON(bytes []byte) error {

	// Unmarshal the string-keyed map
	sk := make(map[string]*Team)
	if err := json.Unmarshal(bytes, &sk); err != nil {
		return err
	}

	// Copy the values
	*im = make(map[TeamUID]*Team)
	for k, v := range sk {
		ki, err := entityuid.ParseEntityUID(k)
		if err != nil {
			return err
		}
		(*im)[TeamUID{EntityUID: ki}] = v
	}

	return nil
}

// ListUIDToListMap is a special type created to help unmarshal map[ListUID]List.
//
// Inspired by https://groups.google.com/g/golang-nuts/c/IxPipKwI-zQ and https://go.dev/play/p/YgUIFxT7hA.
type ListUIDToListMap map[ListUID]*List

func (im *ListUIDToListMap) UnmarshalJSON(bytes []byte) error {

	// Unmarshal the string-keyed map
	sk := make(map[string]*List)
	if err := json.Unmarshal(bytes, &sk); err != nil {
		return err
	}

	// Copy the values
	*im = make(map[ListUID]*List)
	for k, v := range sk {
		ki, err := entityuid.ParseEntityUID(k)
		if err != nil {
			return err
		}
		(*im)[ListUID{EntityUID: ki}] = v
	}

	return nil
}

// TaskUIDToTaskMap is a special type created to help unmarshal map[TaskUID]Task.
//
// Inspired by https://groups.google.com/g/golang-nuts/c/IxPipKwI-zQ and https://go.dev/play/p/YgUIFxT7hA.
type TaskUIDToTaskMap map[TaskUID]*Task

func (im *TaskUIDToTaskMap) UnmarshalJSON(bytes []byte) error {

	// Unmarshal the string-keyed map
	sk := make(map[string]*Task)
	if err := json.Unmarshal(bytes, &sk); err != nil {
		return err
	}

	// Copy the values
	*im = make(map[TaskUID]*Task)
	for k, v := range sk {
		ki, err := entityuid.ParseEntityUID(k)
		if err != nil {
			return err
		}
		(*im)[TaskUID{EntityUID: ki}] = v
	}

	return nil
}
