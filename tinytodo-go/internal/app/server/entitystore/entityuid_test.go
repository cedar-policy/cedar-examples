package entitystore

import (
	"encoding/json"
	"fmt"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/cedar-policy/cedar-go"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"testing"
)

func Test_EntityUID(t *testing.T) {
	t.Run("verify json.Marshaler interface", func(t *testing.T) {
		var m json.Marshaler
		e := NewEntityUID(entitytype.Application, "TinyTodo")
		m = e
		require.NotNil(t, m)
	})
	t.Run("verify json.Unmarshaler interface", func(t *testing.T) {
		var m json.Unmarshaler
		e := NewEntityUID(entitytype.Application, "TinyTodo")
		m = &e
		require.NotNil(t, m)
	})
}

func TestEntityUID_MarshalJSON(t *testing.T) {
	type fields struct {
		EntityUID cedar.EntityUID
	}
	tests := []struct {
		name    string
		fields  fields
		wantErr assert.ErrorAssertionFunc
	}{
		{
			"valid user",
			fields{EntityUID: cedar.NewEntityUID(entitytype.User.String(), "kesha")},
			assert.NoError,
		},
		{
			"valid team",
			fields{EntityUID: cedar.NewEntityUID(entitytype.Team.String(), "temp")},
			assert.NoError,
		},
		{
			"valid application",
			fields{EntityUID: cedar.NewEntityUID(entitytype.Application.String(), "TinyTodo")},
			assert.NoError,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			e := EntityUID{
				EntityUID: tt.fields.EntityUID,
			}
			got, err := e.MarshalJSON()
			if !tt.wantErr(t, err, fmt.Sprintf("MarshalJSON()")) {
				return
			}
			want := []byte(fmt.Sprintf("%q", e.EntityUID.String()))
			assert.Equalf(t, want, got, "MarshalJSON()")

			var gotE EntityUID
			require.NoError(t, json.Unmarshal(got, &gotE))

			assert.Equal(t, gotE, e)
		})
	}
}

func Test_ParseEntityUID(t *testing.T) {
	type args struct {
		euid string
	}
	tests := []struct {
		name    string
		args    args
		want    EntityUID
		wantErr assert.ErrorAssertionFunc
	}{
		{
			name: "valid user ID",
			args: args{euid: "User::\"kesha\""},
			want: EntityUID{
				cedar.EntityUID{
					Type: entitytype.User.String(),
					ID:   "kesha",
				},
			},
			wantErr: assert.NoError,
		},
		{
			name: "valid user ID without quotes",
			args: args{euid: "User::kesha"},
			want: EntityUID{
				cedar.EntityUID{
					Type: entitytype.User.String(),
					ID:   "kesha",
				},
			},
			wantErr: assert.NoError,
		},
		{
			name: "valid team ID",
			args: args{euid: "Team::\"temp\""},
			want: EntityUID{
				cedar.EntityUID{
					Type: entitytype.Team.String(),
					ID:   "temp",
				},
			},
			wantErr: assert.NoError,
		},
		{
			name: "valid application ID",
			args: args{euid: "Application::\"TinyTodo\""},
			want: EntityUID{
				cedar.EntityUID{
					Type: entitytype.Application.String(),
					ID:   "TinyTodo",
				},
			},
			wantErr: assert.NoError,
		},
		{
			name:    "invalid ID",
			args:    args{euid: "::"},
			want:    EntityUID{},
			wantErr: assert.Error,
		},
		{
			name:    "malformed ID",
			args:    args{euid: "Application:\"TinyTodo\""},
			want:    EntityUID{},
			wantErr: assert.Error,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := ParseEntityUID(tt.args.euid)
			if !tt.wantErr(t, err, fmt.Sprintf("ParseEntityUID(%v)", tt.args.euid)) {
				return
			}
			assert.Equalf(t, tt.want, got, "ParseEntityUID(%v)", tt.args.euid)
		})
	}
}
