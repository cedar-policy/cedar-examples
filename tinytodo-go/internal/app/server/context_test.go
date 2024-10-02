package server

import (
	"context"
	"encoding/json"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"os"
	"path"
	"testing"

	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/action"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/cedar-policy/cedar-go"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func readFile(t *testing.T, filename string) []byte {
	t.Helper()
	res, err := os.ReadFile(filename)
	require.NoError(t, err)
	return res
}

func TestServer_isAuthorized(t *testing.T) {

	// read policies

	psFile := readFile(t, path.Join("../../../", "policies.cedar"))
	ps, err := cedar.NewPolicySetFromBytes("policies.cedar", psFile)
	require.NoError(t, err)

	// read entities (will be modified later)

	esFile := readFile(t, path.Join("../../../", "entities.json"))

	es, err := entitystore.New(esFile)
	require.NoError(t, json.Unmarshal(esFile, &es))

	// create server

	s, err := New("", es, ps)
	require.NoError(t, err)

	// extract users

	userAndrew, ok := es.Users[entitystore.UserUID{
		EntityUID: entityuid.New(entitytype.User, "andrew"),
	}]
	require.True(t, ok)

	userAaron, ok := es.Users[entitystore.UserUID{
		EntityUID: entityuid.New(entitytype.User, "aaron"),
	}]
	require.True(t, ok)

	userKesha, ok := es.Users[entitystore.UserUID{
		EntityUID: entityuid.New(entitytype.User, "kesha"),
	}]
	require.True(t, ok)

	// extract teams

	teamInterns, ok := es.Teams[entitystore.TeamUID{
		EntityUID: entityuid.New(entitytype.Team, "interns"),
	}]
	require.True(t, ok)

	t.Run("test policy 0 Action::CreateList allowed", func(t *testing.T) {

		// evaluate decision

		decision, _, err := s.isAuthorized(
			context.Background(),
			userAndrew.EUID.EntityUID,
			action.CreateList,
			ApplicationEntityUID.EUID,
		)
		require.NoError(t, err)
		assert.True(t, decision)
	})

	t.Run("test policy 0 Action::GetLists allowed", func(t *testing.T) {

		// evaluate decision

		decision, _, err := s.isAuthorized(
			context.Background(),
			userAndrew.EUID.EntityUID,
			action.GetLists,
			ApplicationEntityUID.EUID,
		)
		require.NoError(t, err)
		assert.True(t, bool(decision))
	})

	// create new list

	list0UID := es.GetNextListUID()
	list0Readers := es.InsertNextTeam() // readers for list0
	list0Editors := es.InsertNextTeam() // editors for list0

	list0 := entitystore.NewList(
		list0UID,
		"Cedar blog post",
		userAndrew.EUID,
		list0Readers,
		list0Editors,
		nil,
	)

	s.es.Lists[list0UID] = list0

	t.Run("test policy 1 action::GetList allowed", func(t *testing.T) {

		// check preconditions

		require.Contains(t, s.es.Lists, list0UID)

		decision, _, err := s.isAuthorized(
			context.Background(),
			userAndrew.EUID.EntityUID,
			action.GetList,
			list0.UID.EntityUID,
		)
		require.NoError(t, err)
		assert.True(t, decision)
	})

	t.Run("test policy 1 action::GetList disallowed", func(t *testing.T) {

		// check preconditions

		require.Contains(t, s.es.Lists, list0UID)

		// evaluate decision

		decision, _, err := s.isAuthorized(
			context.Background(),
			userAaron.EUID.EntityUID,
			action.GetList,
			list0.UID.EntityUID,
		)
		require.NoError(t, err)
		assert.False(t, decision)
	})

	t.Run("test policy 1 action::GetList disallowed on non-existent list", func(t *testing.T) {

		// check preconditions

		require.Contains(t, s.es.Lists, list0UID)

		// evaluate decision

		decision, _, err := s.isAuthorized(
			context.Background(),
			userAaron.EUID.EntityUID,
			action.GetList,
			entityuid.New(entitytype.List, "non-existent"),
		)
		require.NoError(t, err)
		assert.False(t, decision)
	})

	t.Run("test policy 1 action::CreateTask allowed", func(t *testing.T) {

		// check preconditions

		require.Contains(t, s.es.Lists, list0UID)

		// evaluate decision

		decision, _, err := s.isAuthorized(
			context.Background(),
			userAndrew.EUID.EntityUID,
			action.CreateTask,
			list0.UID.EntityUID,
		)
		require.NoError(t, err)
		assert.True(t, decision)
	})

	t.Run("test policy 1 action::CreateTask disallowed", func(t *testing.T) {

		// check preconditions

		require.Contains(t, s.es.Lists, list0UID)

		// evaluate decision

		decision, _, err := s.isAuthorized(
			context.Background(),
			userAaron.EUID.EntityUID,
			action.CreateTask,
			list0.UID.EntityUID,
		)
		require.NoError(t, err)
		assert.False(t, decision)
	})

	t.Run("test policy 1 action::UpdateTask allowed", func(t *testing.T) {

		// check preconditions

		require.Contains(t, s.es.Lists, list0UID)

		// evaluate decision

		decision, _, err := s.isAuthorized(
			context.Background(),
			userAndrew.EUID.EntityUID,
			action.UpdateTask,
			list0.UID.EntityUID,
		)
		require.NoError(t, err)
		assert.True(t, decision)
	})

	t.Run("test policy 1 action::EditShare allowed", func(t *testing.T) {

		// check preconditions

		require.Contains(t, s.es.Lists, list0UID)

		// evaluate decision

		decision, _, err := s.isAuthorized(
			context.Background(),
			userAndrew.EUID.EntityUID,
			action.EditShare,
			list0.UID.EntityUID,
		)
		require.NoError(t, err)
		assert.True(t, decision)
	})

	// share read permissions with List::list0 with Team::interns

	teamInterns.Parents = append(teamInterns.Parents, list0Readers.EntityUID)

	t.Run("test policy 2 action::GetList allowed", func(t *testing.T) {
		// check preconditions

		require.Contains(t, s.es.Lists, list0UID)

		// evaluate decision

		// User::aaron is a member of Team::interns, which has the parent Team::0 (reader of List::0)

		decision, _, err := s.isAuthorized(
			context.Background(),
			userAaron.EUID.EntityUID,
			action.GetList,
			list0.UID.EntityUID,
		)
		assert.NoError(t, err)
		assert.True(t, decision)
	})

	t.Run("test policy 2 action::UpdateList disallowed", func(t *testing.T) {
		// check preconditions

		require.Contains(t, s.es.Lists, list0UID)

		// evaluate decision

		// User::aaron is a member of Team::interns, which has the parent Team::0 (reader of List::0)

		decision, _, err := s.isAuthorized(
			context.Background(),
			userAaron.EUID.EntityUID,
			action.UpdateList,
			list0.UID.EntityUID,
		)
		assert.NoError(t, err)
		assert.False(t, decision)
	})

	t.Run("test policy 2 action::GetList disallowed", func(t *testing.T) {
		// check preconditions

		require.Contains(t, s.es.Lists, list0UID)

		// evaluate decision

		decisionGet, _, err := s.isAuthorized(
			context.Background(),
			userKesha.EUID.EntityUID,
			action.GetList,
			list0.UID.EntityUID,
		)
		assert.NoError(t, err)
		assert.False(t, decisionGet)

		decisionUpdate, _, err := s.isAuthorized(
			context.Background(),
			userKesha.EUID.EntityUID,
			action.UpdateList,
			list0.UID.EntityUID,
		)
		assert.NoError(t, err)
		assert.False(t, decisionUpdate)
	})

	// share edit permissions with List::list0 with Team::interns

	teamInterns.Parents = append(teamInterns.Parents, list0Editors.EntityUID)

	t.Run("test policy 3 action::{UpdateList, CreateTask, UpdateTask, DeleteTask} allowed", func(t *testing.T) {
		// check preconditions

		require.Contains(t, s.es.Lists, list0UID)

		// evaluate decision1

		// User::aaron is a member of Team::interns, which has the parent Team::1 (editor of List::0)

		decision1, _, err := s.isAuthorized(
			context.Background(),
			userAaron.EUID.EntityUID,
			action.UpdateList,
			list0.UID.EntityUID,
		)
		assert.NoError(t, err)
		assert.True(t, decision1)

		decision2, _, err := s.isAuthorized(
			context.Background(),
			userAaron.EUID.EntityUID,
			action.CreateTask,
			list0.UID.EntityUID,
		)
		assert.NoError(t, err)
		assert.True(t, decision2)

		decision3, _, err := s.isAuthorized(
			context.Background(),
			userAaron.EUID.EntityUID,
			action.UpdateTask,
			list0.UID.EntityUID,
		)
		assert.NoError(t, err)
		assert.True(t, decision3)

		decision4, _, err := s.isAuthorized(
			context.Background(),
			userAaron.EUID.EntityUID,
			action.DeleteTask,
			list0.UID.EntityUID,
		)
		assert.NoError(t, err)
		assert.True(t, decision4)
	})

	t.Run("test policy 3 action::{UpdateList, CreateTask, UpdateTask, DeleteTask} disallowed", func(t *testing.T) {
		// check preconditions

		require.Contains(t, s.es.Lists, list0UID)

		// evaluate decision1

		decision1, _, err := s.isAuthorized(
			context.Background(),
			userKesha.EUID.EntityUID,
			action.UpdateList,
			list0.UID.EntityUID,
		)
		assert.NoError(t, err)
		assert.False(t, decision1)

		decision2, _, err := s.isAuthorized(
			context.Background(),
			userKesha.EUID.EntityUID,
			action.CreateTask,
			list0.UID.EntityUID,
		)
		assert.NoError(t, err)
		assert.False(t, decision2)

		decision3, _, err := s.isAuthorized(
			context.Background(),
			userKesha.EUID.EntityUID,
			action.UpdateTask,
			list0.UID.EntityUID,
		)
		assert.NoError(t, err)
		assert.False(t, decision3)

		decision4, _, err := s.isAuthorized(
			context.Background(),
			userKesha.EUID.EntityUID,
			action.DeleteTask,
			list0.UID.EntityUID,
		)
		assert.NoError(t, err)
		assert.False(t, decision4)
	})

}
