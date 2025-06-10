package server

import (
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entity/app"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"github.com/cedar-policy/cedar-go"
	"log/slog"
	"net"
	"net/http"
	"os"
)

var (
	DefaultLogger = slog.New(
		slog.NewTextHandler(
			os.Stdout,
			&slog.HandlerOptions{
				Level: slog.LevelWarn,
			},
		),
	)
	ApplicationEntityUID = app.App{EUID: entityuid.New(entitytype.Application, "TinyTodo")}
)

// Server represents the web server that host the booking app.
type Server struct {
	addr   string // address to host web server on
	logger *slog.Logger

	// authorization
	es *entitystore.EntityStore
	ps *cedar.PolicySet
}

// Serve starts a HTTP web server.
func (s *Server) Serve() error {

	// we split net.Listen and http.Serve (instead of http.ListenAndServe) because we want to be able to print out
	// the port (if none are specified)

	l, err := net.Listen("tcp", s.addr)
	if err != nil {
		s.logger.Error("failed to listen", slog.Any("err", err))
		return err
	}

	s.logger.Info("starting web server", slog.String("addr", l.Addr().String()))

	return http.Serve(l, s.Routes())
}

// New creates a new Server with the provided address (addr).
//
// addr follows the rules of net.Listen (https://pkg.go.dev/net#Listen).
func New(addr string, es *entitystore.EntityStore, ps *cedar.PolicySet, opts ...Option) (*Server, error) {
	s := &Server{
		addr:   addr,
		logger: DefaultLogger,
		es:     es,
		ps:     ps,
	}
	for _, opt := range opts {
		opt(s)
	}
	return s, nil
}
