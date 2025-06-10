package entitystore

import (
	"encoding/json"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"github.com/cedar-policy/cedar-go/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"testing"
)

func TestEntityStore_AsEntities(t *testing.T) {
	t.Run("basic", func(t *testing.T) {
		f := readFile(t, "../../../../", "entities.json")
		var es EntityStore
		require.NoError(t, json.Unmarshal(f, &es))
		assert.Equal(
			t,
			entityuid.New(entitytype.Application, "TinyTodo"),
			es.App.EUID,
		)

		entities, err := es.AsEntities()
		require.NoError(t, err)

		assert.Contains(
			t,
			entities,
			types.NewEntityUID(
				types.EntityType(entitytype.Application.String()),
				"TinyTodo",
			),
		)

		assert.Contains(
			t,
			entities,
			types.NewEntityUID(
				types.EntityType(entitytype.User.String()),
				"kesha",
			),
		)

		assert.Contains(
			t,
			entities,
			types.NewEntityUID(
				types.EntityType(entitytype.Team.String()),
				"temp",
			),
		)
	})
}
