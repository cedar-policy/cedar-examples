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

func TestList(t *testing.T) {
	t.Run("check interface", func(t *testing.T) {
		var e Entity
		l := NewList(
			ListUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.List.String()),
						ID:   "1",
					},
				},
			},
			"Cedar blog post",
			UserUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.User.String()),
						ID:   "kesha",
					},
				},
			},
			TeamUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.Team.String()),
						ID:   "temp",
					},
				},
			},
			TeamUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.Team.String()),
						ID:   "admin",
					},
				},
			},
			nil,
		)
		e = l
		require.NotNil(t, e)
	})
}

func TestList_Marshal(t *testing.T) {
	t.Run("check marshal valid case", func(t *testing.T) {
		list := NewList(
			ListUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.List.String()),
						ID:   "1",
					},
				},
			},
			"Cedar blog post",
			UserUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.User.String()),
						ID:   "kesha",
					},
				},
			},
			TeamUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.Team.String()),
						ID:   "temp",
					},
				},
			},
			TeamUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.Team.String()),
						ID:   "admin",
					},
				},
			},
			nil,
		)

		got, err := json.MarshalIndent(list, "", "  ")
		require.NoError(t, err)

		var recovered List
		require.NoError(t, json.Unmarshal(got, &recovered))

		assert.Equal(t, *list, recovered)
	})
}
