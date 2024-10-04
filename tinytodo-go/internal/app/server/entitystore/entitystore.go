// Package entitystore contains the definitions for entities encountered in TinyTodo.
//
// It is translated from entitystore.rs.
package entitystore

import (
	"encoding/json"
	"fmt"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entity/app"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entity/list"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entity/task"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entity/team"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entity/user"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"strconv"
)

// EntityStore tracks all entities that are handled in TinyTodo.
type EntityStore struct {
	Users UserUIDToUserMap `json:"users"`
	Teams TeamUIDToTeamMap `json:"teams"`
	Lists ListUIDToListMap `json:"lists"`
	App   app.App          `json:"app"`
}

func New(entitiesJSON []byte) (*EntityStore, error) {
	var es EntityStore
	if err := json.Unmarshal(entitiesJSON, &es); err != nil {
		return nil, fmt.Errorf("failed to unmarshal EntityStore: %w", err)
	}
	return &es, nil
}

// GetNextListUID returns the next available ListUID (but does not create) with a monotonically increasing ID.
func (e *EntityStore) GetNextListUID() list.ListUID {
	var id int
	for {
		listUID := list.ListUID{
			EntityUID: entityuid.New(entitytype.List, strconv.Itoa(id)),
		}
		if _, found := e.Lists[listUID]; !found {
			return listUID
		}
		id++
	}
}

// InsertNextTeam creates the next available Team with a monotonically increasing ID and returns the TeamUID.
func (e *EntityStore) InsertNextTeam() team.TeamUID {
	var id int
	for {
		teamUID := team.TeamUID{
			EntityUID: entityuid.New(entitytype.Team, strconv.Itoa(id)),
		}
		if _, found := e.Teams[teamUID]; !found {
			e.Teams[teamUID] = team.New(teamUID, nil)
			return teamUID
		}
		id++
	}
}

// UserUIDToUserMap is a special type created to help unmarshal map[UserUID]User.
//
// Inspired by https://groups.google.com/g/golang-nuts/c/IxPipKwI-zQ and https://go.dev/play/p/YgUIFxT7hA.
type UserUIDToUserMap map[user.UserUID]*user.User

func (im *UserUIDToUserMap) UnmarshalJSON(bytes []byte) error {

	// Unmarshal the string-keyed map
	sk := make(map[string]*user.User)
	if err := json.Unmarshal(bytes, &sk); err != nil {
		return err
	}

	// Copy the values
	*im = make(map[user.UserUID]*user.User)
	for k, v := range sk {
		ki, err := entityuid.Parse(k)
		if err != nil {
			return err
		}
		(*im)[user.UserUID{
			EntityUID: ki,
		}] = v
	}

	return nil
}

// TeamUIDToTeamMap is a special type created to help unmarshal map[TeamUID]Team.
//
// Inspired by https://groups.google.com/g/golang-nuts/c/IxPipKwI-zQ and https://go.dev/play/p/YgUIFxT7hA.
type TeamUIDToTeamMap map[team.TeamUID]*team.Team

func (im *TeamUIDToTeamMap) UnmarshalJSON(bytes []byte) error {

	// Unmarshal the string-keyed map
	sk := make(map[string]*team.Team)
	if err := json.Unmarshal(bytes, &sk); err != nil {
		return err
	}

	// Copy the values
	*im = make(map[team.TeamUID]*team.Team)
	for k, v := range sk {
		ki, err := entityuid.Parse(k)
		if err != nil {
			return err
		}
		(*im)[team.TeamUID{EntityUID: ki}] = v
	}

	return nil
}

// ListUIDToListMap is a special type created to help unmarshal map[ListUID]List.
//
// Inspired by https://groups.google.com/g/golang-nuts/c/IxPipKwI-zQ and https://go.dev/play/p/YgUIFxT7hA.
type ListUIDToListMap map[list.ListUID]*list.List

func (im *ListUIDToListMap) UnmarshalJSON(bytes []byte) error {

	// Unmarshal the string-keyed map
	sk := make(map[string]*list.List)
	if err := json.Unmarshal(bytes, &sk); err != nil {
		return err
	}

	// Copy the values
	*im = make(map[list.ListUID]*list.List)
	for k, v := range sk {
		ki, err := entityuid.Parse(k)
		if err != nil {
			return err
		}
		(*im)[list.ListUID{EntityUID: ki}] = v
	}

	return nil
}

// TaskUIDToTaskMap is a special type created to help unmarshal map[TaskUID]Task.
//
// Inspired by https://groups.google.com/g/golang-nuts/c/IxPipKwI-zQ and https://go.dev/play/p/YgUIFxT7hA.
type TaskUIDToTaskMap map[task.TaskUID]*task.Task

func (im *TaskUIDToTaskMap) UnmarshalJSON(bytes []byte) error {

	// Unmarshal the string-keyed map
	sk := make(map[string]*task.Task)
	if err := json.Unmarshal(bytes, &sk); err != nil {
		return err
	}

	// Copy the values
	*im = make(map[task.TaskUID]*task.Task)
	for k, v := range sk {
		ki, err := entityuid.Parse(k)
		if err != nil {
			return err
		}
		(*im)[task.TaskUID{EntityUID: ki}] = v
	}

	return nil
}
