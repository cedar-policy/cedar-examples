package server

import (
	"github.com/go-chi/chi/v5"
	"net/http"

	"github.com/go-chi/chi/v5/middleware"
)

func (s *Server) Routes() http.Handler {

	mux := chi.NewRouter()

	// ----- [] define middleware

	mux.Use(middleware.Recoverer)
	mux.Use(s.logRequest)

	// ----- [] define routes

	mux.Get("/", s.handleGetHome)
	mux.Get("/api/lists/get", s.handleGetAPIListsGet)
	mux.Post("/api/list/create", s.handlePostAPIListCreate)
	mux.Get("/api/list/get", s.handleGetAPIListGet)
	mux.Post("/api/task/create", s.handlePostAPITaskCreate)
	mux.Post("/api/task/update", s.handlePostAPITaskUpdate)
	mux.Delete("/api/task/delete", s.handleDeleteAPITaskDelete)
	mux.Delete("/api/list/delete", s.handleDeleteAPIListDelete)
	mux.Post("/api/share", s.handlePostAPIShare)
	mux.Delete("/api/share", s.handleDeleteAPIShare)

	return mux
}
