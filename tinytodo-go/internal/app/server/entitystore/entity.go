package entitystore

import (
	"github.com/cedar-policy/cedar-go/types"
)

type Entity interface {
	AsCedarEntity() *types.Entity
}
