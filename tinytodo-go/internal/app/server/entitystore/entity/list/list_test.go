package list

import (
	"encoding/json"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entity"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entity/team"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entity/user"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"github.com/cedar-policy/cedar-go/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"testing"
)

func TestList(t *testing.T) {
	t.Run("check interface", func(t *testing.T) {
		var e entity.Entity
		l := New(
			ListUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.List.String()),
						ID:   "1",
					},
				},
			},
			"Cedar blog post",
			user.UserUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.User.String()),
						ID:   "kesha",
					},
				},
			},
			team.TeamUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.Team.String()),
						ID:   "temp",
					},
				},
			},
			team.TeamUID{
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
		list := New(
			ListUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.List.String()),
						ID:   "1",
					},
				},
			},
			"Cedar blog post",
			user.UserUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.User.String()),
						ID:   "kesha",
					},
				},
			},
			team.TeamUID{
				EntityUID: entityuid.EntityUID{
					EntityUID: types.EntityUID{
						Type: types.EntityType(entitytype.Team.String()),
						ID:   "temp",
					},
				},
			},
			team.TeamUID{
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
