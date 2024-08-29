package server

import (
	"encoding/json"
	"fmt"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/action"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/role"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/taskstate"
	"log/slog"
	"net/http"
	"strconv"
)

// handleGetHome handles GET requests to "/".
func (s *Server) handleGetHome(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)
}

// handleGetAPIListsGet handles GET requests to "/api/lists/get".
//
// Example response:
//
//	[
//		{'uid': 'List::"0"', 'owner': 'User::"andrew"', 'name': 'Cedar blog post', 'tasks': [], 'readers': 'Team::"1"', 'editors': 'Team::"2"'},
//		{'uid': 'List::"1"', 'owner': 'User::"aaron"', 'name': 'my task list', 'tasks': [], 'readers': 'Team::"3"', 'editors': 'Team::"4"'}
//	]
func (s *Server) handleGetAPIListsGet(w http.ResponseWriter, r *http.Request) {

	userUID_, err := entitystore.ParseEntityUID(r.URL.Query().Get("uid"))
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process uid",
			slog.Any("error", err),
			slog.String("uid", r.URL.Query().Get("uid")),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process uid",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID := entitystore.UserUID{EntityUID: userUID_}

	s.logger.InfoContext(
		r.Context(),
		"processing get lists request",
		slog.Any("userUID", userUID),
	)

	decision, diagnostic, err := s.isAuthorized(
		r.Context(),
		userUID.EntityUID,
		action.GetLists,
		ApplicationEntityUID.EUID,
	)
	if err != nil {
		s.logger.ErrorContext(
			r.Context(),
			"failed to check authorization",
			slog.Any("principal", userUID),
			slog.Any("action", action.GetLists),
			slog.Any("resource", ApplicationEntityUID),
			slog.Any("error", err),
		)
		s.ServerError(w, err)
		return
	}

	if !decision {
		s.logger.InfoContext(
			r.Context(),
			"not allowed",
			slog.Any("principal", userUID),
			slog.Any("action", action.GetLists),
			slog.Any("resource", ApplicationEntityUID),
			slog.Any("diagnostic", diagnostic),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Authorization Denied", // this particular error message is required by tinytodo.py
			},
			http.StatusOK, // tinytodo.py expects 200 for authorization denied
		)
		return
	}

	// client expects allowedLists to be an empty list (vs nil), don't convert to var declaration
	allowedLists := []*entitystore.List{}

	for listEUID, list := range s.es.Lists {
		listAllowed, _, err := s.isAuthorized(
			r.Context(),
			userUID.EntityUID,
			action.GetList,
			listEUID.EntityUID,
		)
		if err != nil {
			s.logger.ErrorContext(
				r.Context(),
				"failed to check authorization",
				slog.Any("userUID", userUID),
				slog.Any("listEUID", listEUID),
				slog.Any("error", err),
			)
			s.ServerError(w, err)
			return
		}
		if !listAllowed {
			continue
		}
		allowedLists = append(allowedLists, list)
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(allowedLists)
}

type apiListCreateRequest struct {
	UID  string `json:"uid"`
	Name string `json:"name"`
}

// handlePostAPIListCreate handles POST requests to "/api/list/create".
//
// Example response:
//
//	List::"0"
func (s *Server) handlePostAPIListCreate(w http.ResponseWriter, r *http.Request) {
	var req apiListCreateRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process request body",
			slog.Any("error", err),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process request body",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID_, err := entitystore.ParseEntityUID(req.UID)
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process uid",
			slog.Any("error", err),
			slog.String("uid", req.UID),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process uid",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID := entitystore.UserUID{EntityUID: userUID_}

	s.logger.InfoContext(
		r.Context(),
		"processing create list request",
		slog.Any("req", req),
		slog.Any("userUID", userUID),
	)

	decision, diagnostic, err := s.isAuthorized(
		r.Context(),
		userUID.EntityUID,
		action.CreateList,
		ApplicationEntityUID.EUID,
	)
	if err != nil {
		s.logger.ErrorContext(
			r.Context(),
			"failed to check authorization",
			slog.Any("principal", userUID),
			slog.Any("action", action.CreateList),
			slog.Any("resource", ApplicationEntityUID),
			slog.Any("error", err),
		)
		s.ServerError(w, err)
		return
	}

	if !decision {
		s.logger.InfoContext(
			r.Context(),
			"not allowed",
			slog.Any("principal", userUID),
			slog.Any("action", action.CreateList),
			slog.Any("resource", ApplicationEntityUID),
			slog.Any("diagnostic", diagnostic),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Authorization Denied", // this particular error message is required by tinytodo.py
			},
			http.StatusOK, // tinytodo.py expects 200 for authorization denied
		)
		return
	}

	listUID := s.es.GetNextListUID()
	newReader := s.es.InsertNextTeam()
	newEditor := s.es.InsertNextTeam()

	s.logger.InfoContext(
		r.Context(),
		"creating list",
		slog.Any("listUID", listUID),
		slog.Any("name", req.Name),
		slog.Any("owner", userUID),
		slog.Any("newReader", newReader),
		slog.Any("newEditor", newEditor),
	)

	s.es.Lists[listUID] = entitystore.NewList(
		listUID,
		req.Name,
		userUID,
		newReader,
		newEditor,
		nil, // will be converted to empty slice later
	)

	w.Header().Set("Content-Type", "application/json")
	w.Write([]byte(fmt.Sprintf("%q", listUID.String())))
}

// handleGetAPIListGet handles GET requests to "/api/list/get".
//
// Example response:
//
//	{'uid': 'List::"0"', 'owner': 'User::"andrew"', 'name': 'a', 'tasks': [], 'readers': 'Team::"1"', 'editors': 'Team::"2"'}
func (s *Server) handleGetAPIListGet(w http.ResponseWriter, r *http.Request) {
	userUID_, err := entitystore.ParseEntityUID(r.URL.Query().Get("uid"))
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process uid",
			slog.Any("error", err),
			slog.String("uid", r.URL.Query().Get("uid")),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process uid",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID := entitystore.UserUID{EntityUID: userUID_}

	listUID_, err := entitystore.ParseEntityUID(r.URL.Query().Get("list"))
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process list",
			slog.Any("error", err),
			slog.String("list", r.URL.Query().Get("list")),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process list",
			},
			http.StatusBadRequest,
		)
		return
	}

	listUID := entitystore.ListUID{EntityUID: listUID_}

	s.logger.InfoContext(
		r.Context(),
		"processing get list request",
		slog.Any("userUID", userUID),
		slog.Any("listUID", listUID),
	)

	decision, diagnostic, err := s.isAuthorized(
		r.Context(),
		userUID.EntityUID,
		action.GetList,
		listUID.EntityUID,
	)
	if err != nil {
		s.logger.ErrorContext(
			r.Context(),
			"failed to check authorization",
			slog.Any("principal", userUID),
			slog.Any("action", action.GetList),
			slog.Any("resource", listUID),
			slog.Any("error", err),
		)
		s.ServerError(w, err)
		return
	}

	if !decision {
		s.logger.InfoContext(
			r.Context(),
			"not allowed",
			slog.Any("principal", userUID),
			slog.Any("action", action.GetList),
			slog.Any("resource", listUID),
			slog.Any("diagnostic", diagnostic),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Authorization Denied", // this particular error message is required by tinytodo.py
			},
			http.StatusOK, // tinytodo.py expects 200 for authorization denied
		)
		return
	}

	// no need to check if list exists -- non-existent lists will automatically be forbidden

	list := s.es.Lists[listUID]

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(list)

}

type apiTaskCreateRequest struct {
	UID  string `json:"uid"`
	List string `json:"list"`
	Name string `json:"name"`
}

// handlePostAPITaskCreate handles POST requests to "/api/task/create".
//
// Returns the ID of the new task.
//
// Example response:
//
//	0
func (s *Server) handlePostAPITaskCreate(w http.ResponseWriter, r *http.Request) {
	var req apiTaskCreateRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process request body",
			slog.Any("error", err),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process request body",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID_, err := entitystore.ParseEntityUID(req.UID)
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process uid",
			slog.Any("error", err),
			slog.String("uid", req.UID),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process uid",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID := entitystore.UserUID{EntityUID: userUID_}

	listUID_, err := entitystore.ParseEntityUID(req.List)
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process list",
			slog.Any("error", err),
			slog.String("list", req.List),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process list",
			},
			http.StatusBadRequest,
		)
		return
	}

	listUID := entitystore.ListUID{EntityUID: listUID_}

	s.logger.InfoContext(
		r.Context(),
		"processing create task request",
		slog.Any("req", req),
		slog.Any("userUID", userUID),
		slog.Any("listUID", listUID),
	)

	decision, diagnostic, err := s.isAuthorized(
		r.Context(),
		userUID.EntityUID,
		action.CreateTask,
		listUID.EntityUID,
	)
	if err != nil {
		s.logger.ErrorContext(
			r.Context(),
			"failed to check authorization",
			slog.Any("principal", userUID),
			slog.Any("action", action.CreateTask),
			slog.Any("resource", listUID),
			slog.Any("error", err),
		)
		s.ServerError(w, err)
		return
	}

	if !decision {
		s.logger.InfoContext(
			r.Context(),
			"not allowed",
			slog.Any("principal", userUID),
			slog.Any("action", action.CreateTask),
			slog.Any("resource", listUID),
			slog.Any("diagnostic", diagnostic),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Authorization Denied", // this particular error message is required by tinytodo.py
			},
			http.StatusOK, // tinytodo.py expects 200 for authorization denied
		)
		return
	}

	// no need to check if list exists -- non-existent lists will automatically be forbidden

	list, ok := s.es.Lists[listUID]
	if !ok {
		s.logger.ErrorContext(
			r.Context(),
			"failed to retrieve list -"+
				" this shouldn't happen because Cedar should have checked that the list exists",
			slog.Any("userUID", userUID),
			slog.Any("error", err),
		)
		s.ServerError(w, err)
		return
	}

	id := list.InsertTask(req.Name)

	s.logger.InfoContext(
		r.Context(),
		"created task",
		slog.Any("listUID", listUID),
		slog.String("name", req.Name),
		slog.Any("owner", userUID),
		slog.Int("id", id),
	)

	w.Write([]byte(strconv.Itoa(id))) // hackish
}

type apiTaskUpdateRequest struct {
	UID   string `json:"uid"`
	List  string `json:"list"`
	Task  int    `json:"task"`
	State string `json:"state"`
}

// handlePostAPITaskUpdate handles POST requests to "/api/task/update".
//
// Example response:
//
//	{'message': 'ok'}
func (s *Server) handlePostAPITaskUpdate(w http.ResponseWriter, r *http.Request) {

	var req apiTaskUpdateRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process request body",
			slog.Any("error", err),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process request body",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID_, err := entitystore.ParseEntityUID(req.UID)
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process uid",
			slog.Any("error", err),
			slog.String("uid", req.UID),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process uid",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID := entitystore.UserUID{EntityUID: userUID_}

	listUID_, err := entitystore.ParseEntityUID(req.List)
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process list",
			slog.Any("error", err),
			slog.String("list", req.List),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process list",
			},
			http.StatusBadRequest,
		)
		return
	}

	listUID := entitystore.ListUID{EntityUID: listUID_}

	taskState := taskstate.Parse(req.State)
	if taskState == taskstate.Unknown {
		s.logger.InfoContext(
			r.Context(),
			"invalid task state",
			slog.String("state", req.State),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Invalid task state",
			},
			http.StatusBadRequest,
		)
		return
	}

	s.logger.InfoContext(
		r.Context(),
		"processing update task request",
		slog.Any("req", req),
		slog.Any("userUID", userUID),
		slog.Any("listUID", listUID),
		slog.Any("taskState", taskState),
	)

	decision, diagnostic, err := s.isAuthorized(
		r.Context(),
		userUID.EntityUID,
		action.UpdateTask,
		listUID.EntityUID, // note that we check for permission to list, not task
	)
	if err != nil {
		s.logger.ErrorContext(
			r.Context(),
			"failed to check authorization",
			slog.Any("principal", userUID),
			slog.Any("action", action.UpdateTask),
			slog.Any("resource", listUID),
			slog.Any("error", err),
		)
		s.ServerError(w, err)
		return
	}

	if !decision {
		s.logger.InfoContext(
			r.Context(),
			"not allowed",
			slog.Any("principal", userUID),
			slog.Any("action", action.UpdateTask),
			slog.Any("resource", listUID),
			slog.Any("diagnostic", diagnostic),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Authorization Denied", // this particular error message is required by tinytodo.py
			},
			http.StatusOK, // tinytodo.py expects 200 for authorization denied
		)
		return
	}

	// no need to check if list exists -- non-existent lists will automatically be forbidden

	s.logger.InfoContext(
		r.Context(),
		"updating task",
		slog.Any("req", req),
		slog.Any("userUID", userUID),
		slog.Any("listUID", listUID),
		slog.Any("taskState", taskState),
	)

	list, ok := s.es.Lists[listUID]
	if !ok {
		s.logger.ErrorContext(
			r.Context(),
			"failed to retrieve list -"+
				" this shouldn't happen because Cedar should have checked that the list exists",
			slog.Any("userUID", userUID),
			slog.Any("error", err),
		)
		s.ServerError(w, err)
		return
	}

	if req.Task < 0 || req.Task >= len(list.Tasks) {
		s.logger.InfoContext(
			r.Context(),
			fmt.Sprintf("The list %s does not contain a task with ID %d", listUID, req.Task),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: fmt.Sprintf(
					"The list %s does not contain a task with ID %d",
					listUID,
					req.Task,
				), // this particular error message is required by tinytodo.py
			},
			http.StatusOK, // tinytodo.py expects 200 for authorization denied
		)
		return
	}

	task := list.Tasks[req.Task]
	task.State = taskState

	s.writeGenericResponse(
		r.Context(),
		w,
		&GenericResponse{
			Message: "ok", // this particular message is required by tinytodo.py
		},
		http.StatusOK,
	)
	return
}

type apiTaskDeleteRequest struct {
	UID  string `json:"uid"`
	List string `json:"list"`
	Task int    `json:"task"`
}

// handleDeleteAPITaskDelete handles DELETE requests to "/api/task/delete".
//
// Example response:
//
//	{'message': 'ok'}
func (s *Server) handleDeleteAPITaskDelete(w http.ResponseWriter, r *http.Request) {
	var req apiTaskDeleteRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process request body",
			slog.Any("error", err),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process request body",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID_, err := entitystore.ParseEntityUID(req.UID)
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process uid",
			slog.Any("error", err),
			slog.String("uid", req.UID),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process uid",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID := entitystore.UserUID{EntityUID: userUID_}

	listUID_, err := entitystore.ParseEntityUID(req.List)
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process list",
			slog.Any("error", err),
			slog.String("list", req.List),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process list",
			},
			http.StatusBadRequest,
		)
		return
	}

	listUID := entitystore.ListUID{EntityUID: listUID_}

	s.logger.InfoContext(
		r.Context(),
		"processing delete task request",
		slog.Any("req", req),
		slog.Any("userUID", userUID),
		slog.Any("listUID", listUID),
	)

	decision, diagnostic, err := s.isAuthorized(
		r.Context(),
		userUID.EntityUID,
		action.DeleteTask,
		listUID.EntityUID,
	)
	if err != nil {
		s.logger.ErrorContext(
			r.Context(),
			"failed to check authorization",
			slog.Any("principal", userUID),
			slog.Any("action", action.DeleteTask),
			slog.Any("resource", listUID),
			slog.Any("err", err),
		)
		s.ServerError(w, err)
		return
	}

	if !decision {
		s.logger.InfoContext(
			r.Context(),
			"not allowed",
			slog.Any("principal", userUID),
			slog.Any("action", action.DeleteTask),
			slog.Any("resource", listUID),
			slog.Any("diagnostic", diagnostic),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Authorization Denied", // this particular error message is required by tinytodo.py
			},
			http.StatusOK, // tinytodo.py expects 200 for authorization denied
		)
		return
	}

	// no need to check if list exists -- non-existent lists will automatically be forbidden

	s.logger.InfoContext(
		r.Context(),
		"deleting task",
		slog.Any("req", req),
		slog.Any("userUID", userUID),
		slog.Any("listUID", listUID),
		slog.Any("task", req.Task),
	)

	list, ok := s.es.Lists[listUID]
	if !ok {
		s.logger.ErrorContext(
			r.Context(),
			"failed to retrieve list -"+
				" this shouldn't happen because Cedar should have checked that the list exists",
			slog.Any("userUID", userUID),
			slog.Any("error", err),
		)
		s.ServerError(w, err)
		return
	}

	if req.Task < 0 || req.Task >= len(list.Tasks) {
		s.logger.InfoContext(
			r.Context(),
			fmt.Sprintf("The list %s does not contain a task with ID %d", listUID, req.Task),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: fmt.Sprintf(
					"The list %s does not contain a task with ID %d",
					listUID,
					req.Task,
				), // this particular error message is required by tinytodo.py
			},
			http.StatusOK, // tinytodo.py expects 200 for authorization denied
		)
		return
	}

	list.DeleteTask(req.Task)

	s.writeGenericResponse(
		r.Context(),
		w,
		&GenericResponse{
			Message: "ok", // this particular message is required by tinytodo.py
		},
		http.StatusOK,
	)
	return
}

type apiListDeleteRequest struct {
	UID  string `json:"uid"`
	List string `json:"list"`
}

// handleDeleteAPIListDelete handles DELETE requests to "/api/list/delete".
//
// Example response:
//
//	{'message': 'ok'}
func (s *Server) handleDeleteAPIListDelete(w http.ResponseWriter, r *http.Request) {
	var req apiListDeleteRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process request body",
			slog.Any("error", err),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process request body",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID_, err := entitystore.ParseEntityUID(req.UID)
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process uid",
			slog.Any("error", err),
			slog.String("uid", req.UID),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process uid",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID := entitystore.UserUID{EntityUID: userUID_}

	listUID_, err := entitystore.ParseEntityUID(req.List)
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process list",
			slog.Any("error", err),
			slog.String("list", req.List),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process list",
			},
			http.StatusBadRequest,
		)
		return
	}

	listUID := entitystore.ListUID{EntityUID: listUID_}

	s.logger.InfoContext(
		r.Context(),
		"processing delete list request",
		slog.Any("req", req),
		slog.Any("userUID", userUID),
		slog.Any("listUID", listUID),
	)

	decision, diagnostic, err := s.isAuthorized(
		r.Context(),
		userUID.EntityUID,
		action.DeleteList,
		listUID.EntityUID,
	)
	if err != nil {
		s.logger.ErrorContext(
			r.Context(),
			"failed to check authorization",
			slog.Any("principal", userUID),
			slog.Any("action", action.DeleteList),
			slog.Any("resource", listUID),
			slog.Any("err", err),
		)
		s.ServerError(w, err)
		return
	}

	if !decision {
		s.logger.InfoContext(
			r.Context(),
			"not allowed",
			slog.Any("principal", userUID),
			slog.Any("action", action.DeleteList),
			slog.Any("resource", listUID),
			slog.Any("diagnostic", diagnostic),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Authorization Denied", // this particular error message is required by tinytodo.py
			},
			http.StatusOK, // tinytodo.py expects 200 for authorization denied
		)
		return
	}

	// no need to check if list exists -- non-existent lists will automatically be forbidden

	s.logger.InfoContext(
		r.Context(),
		"deleting list",
		slog.Any("req", req),
		slog.Any("userUID", userUID),
		slog.Any("listUID", listUID),
	)

	delete(s.es.Lists, listUID)

	s.writeGenericResponse(
		r.Context(),
		w,
		&GenericResponse{
			Message: "ok", // this particular message is required by tinytodo.py
		},
		http.StatusOK,
	)
	return
}

type apiShareRequest struct {
	UID       string `json:"uid"`
	List      string `json:"list"`
	Role      string `json:"role"`       // Reader or Editor
	ShareWith string `json:"share_with"` // UserUID or TeamUID to share with
}

// handlePostAPIShare handles POST requests to "/api/share".
//
// Example response:
//
//	{'message': 'ok'}
func (s *Server) handlePostAPIShare(w http.ResponseWriter, r *http.Request) {
	var req apiShareRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process request body",
			slog.Any("error", err),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process request body",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID_, err := entitystore.ParseEntityUID(req.UID)
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process uid",
			slog.Any("error", err),
			slog.String("uid", req.UID),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process uid",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID := entitystore.UserUID{EntityUID: userUID_}

	listUID_, err := entitystore.ParseEntityUID(req.List)
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process list",
			slog.Any("error", err),
			slog.String("list", req.List),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process list",
			},
			http.StatusBadRequest,
		)
		return
	}

	listUID := entitystore.ListUID{EntityUID: listUID_}

	shareWith, err := entitystore.ParseEntityUID(req.ShareWith) // can be User or Team
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process share_with",
			slog.Any("error", err),
			slog.String("share_with", req.ShareWith),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process share_with",
			},
			http.StatusBadRequest,
		)
		return
	}

	rr := role.Parse(req.Role)
	if rr == role.Unknown {
		s.logger.InfoContext(
			r.Context(),
			"invalid role",
			slog.Any("req", req),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: fmt.Sprintf("Invalid role: %q", req.Role),
			},
			http.StatusBadRequest,
		)
		return
	}

	s.logger.InfoContext(
		r.Context(),
		"processing share list request",
		slog.Any("req", req),
		slog.Any("userUID", userUID),
		slog.Any("listUID", listUID),
		slog.Any("shareWith", shareWith),
		slog.Any("role", rr),
	)

	decision, diagnostic, err := s.isAuthorized(
		r.Context(),
		userUID.EntityUID,
		action.EditShare,
		listUID.EntityUID,
	)
	if err != nil {
		s.logger.ErrorContext(
			r.Context(),
			"failed to check authorization",
			slog.Any("principal", userUID),
			slog.Any("action", action.EditShare),
			slog.Any("resource", listUID),
			slog.Any("error", err),
		)
		s.ServerError(w, err)
		return
	}

	if !decision {
		s.logger.InfoContext(
			r.Context(),
			"not allowed",
			slog.Any("principal", userUID),
			slog.Any("action", action.EditShare),
			slog.Any("resource", listUID),
			slog.Any("diagnostic", diagnostic),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Authorization Denied", // this particular error message is required by tinytodo.py
			},
			http.StatusOK, // tinytodo.py expects 200 for authorization denied
		)
		return
	}

	// no need to check if list exists -- non-existent lists will automatically be forbidden

	list := s.es.Lists[listUID]

	var roleTeamUID entitystore.TeamUID

	switch rr {
	case role.Editor:
		roleTeamUID = list.Editors
	case role.Reader:
		roleTeamUID = list.Readers
	default:
		// should have already checked for valid role
		s.logger.ErrorContext(
			r.Context(),
			"invalid role",
			slog.Any("req", req),
		)
		s.ServerError(w, err)
		return
	}

	if shareWithTeam, ok := s.es.Teams[entitystore.TeamUID{EntityUID: shareWith}]; ok {
		s.logger.InfoContext(
			r.Context(),
			"found share_with Team entity",
			slog.Any("shareWithTeam", shareWithTeam),
		)
		shareWithTeam.Parents = append(shareWithTeam.Parents, roleTeamUID.EntityUID)
	} else if shareWithUser, ok := s.es.Users[entitystore.UserUID{EntityUID: shareWith}]; ok {
		s.logger.InfoContext(
			r.Context(),
			"found share_with User entity",
			slog.Any("shareWithUser", shareWithUser),
		)
		shareWithUser.Parents = append(shareWithUser.Parents, roleTeamUID.EntityUID)
	} else {
		s.logger.InfoContext(
			r.Context(),
			"failed to find share_with entity",
			slog.String("share_with", req.ShareWith),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to find share_with entity",
			},
			http.StatusBadRequest,
		)
		return
	}

	s.writeGenericResponse(
		r.Context(),
		w,
		&GenericResponse{
			Message: "ok", // this particular message is required by tinytodo.py
		},
		http.StatusOK,
	)
	return
}

type apiUnshareRequest struct {
	UID         string `json:"uid"`
	List        string `json:"list"`
	Role        string `json:"role"`         // Reader or Editor
	UnshareWith string `json:"unshare_with"` // UserUID or TeamUID to unshare with
}

func removeParent(parents []entitystore.EntityUID, remove entitystore.EntityUID) []entitystore.EntityUID {

	found := -1

	for i, p := range parents {
		if p == remove {
			found = i
			break
		}
	}

	if found > 0 {
		// remove the parent
		parents = append(parents[:found], parents[found+1:]...)
	}

	return parents
}

// handleDeleteAPIShare handles DELETE requests to "/api/share".
//
// Example response:
//
//	{'message': 'ok'}
func (s *Server) handleDeleteAPIShare(w http.ResponseWriter, r *http.Request) {
	var req apiUnshareRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process request body",
			slog.Any("error", err),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process request body",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID_, err := entitystore.ParseEntityUID(req.UID)
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process uid",
			slog.Any("error", err),
			slog.String("uid", req.UID),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process uid",
			},
			http.StatusBadRequest,
		)
		return
	}

	userUID := entitystore.UserUID{EntityUID: userUID_}

	listUID_, err := entitystore.ParseEntityUID(req.List)
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process list",
			slog.Any("error", err),
			slog.String("list", req.List),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process list",
			},
			http.StatusBadRequest,
		)
		return
	}

	listUID := entitystore.ListUID{EntityUID: listUID_}

	unshareWith, err := entitystore.ParseEntityUID(req.UnshareWith) // can be User or Team
	if err != nil {
		s.logger.InfoContext(
			r.Context(),
			"failed to process share_with",
			slog.Any("error", err),
			slog.String("unshare_with", req.UnshareWith),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to process unshare_with",
			},
			http.StatusBadRequest,
		)
		return
	}

	rr := role.Parse(req.Role)
	if rr == role.Unknown {
		s.logger.InfoContext(
			r.Context(),
			"invalid role",
			slog.Any("req", req),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: fmt.Sprintf("Invalid role: %q", req.Role),
			},
			http.StatusBadRequest,
		)
		return
	}

	s.logger.InfoContext(
		r.Context(),
		"processing unshare list request",
		slog.Any("req", req),
		slog.Any("userUID", userUID),
		slog.Any("listUID", listUID),
		slog.Any("unshareWith", unshareWith),
		slog.Any("role", rr),
	)

	decision, diagnostic, err := s.isAuthorized(
		r.Context(),
		userUID.EntityUID,
		action.EditShare,
		listUID.EntityUID,
	)
	if err != nil {
		s.logger.ErrorContext(
			r.Context(),
			"failed to check authorization",
			slog.Any("principal", userUID),
			slog.Any("action", action.EditShare),
			slog.Any("resource", listUID),
			slog.Any("error", err),
		)
		s.ServerError(w, err)
		return
	}

	if !decision {
		s.logger.InfoContext(
			r.Context(),
			"not allowed",
			slog.Any("principal", userUID),
			slog.Any("action", action.EditShare),
			slog.Any("resource", listUID),
			slog.Any("diagnostic", diagnostic),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Authorization Denied", // this particular error message is required by tinytodo.py
			},
			http.StatusOK, // tinytodo.py expects 200 for authorization denied
		)
		return
	}

	// no need to check if list exists -- non-existent lists will automatically be forbidden

	list := s.es.Lists[listUID]

	var roleTeamUID entitystore.TeamUID

	switch rr {
	case role.Editor:
		roleTeamUID = list.Editors
	case role.Reader:
		roleTeamUID = list.Readers
	default:
		// should have already checked for valid role
		s.logger.ErrorContext(
			r.Context(),
			"invalid role",
			slog.Any("req", req),
		)
		s.ServerError(w, err)
		return
	}

	if unshareWithTeam, ok := s.es.Teams[entitystore.TeamUID{EntityUID: unshareWith}]; ok {
		s.logger.InfoContext(
			r.Context(),
			"found unshare_with Team entity",
			slog.Any("unshareWithTeam", unshareWithTeam),
		)
		unshareWithTeam.Parents = removeParent(unshareWithTeam.Parents, roleTeamUID.EntityUID)
		s.logger.InfoContext(
			r.Context(),
			"remove parent from unshare_with Team entity",
			slog.Any("unshareWithTeam", unshareWithTeam),
			slog.Any("removed", roleTeamUID),
		)
	} else if unshareWithUser, ok := s.es.Users[entitystore.UserUID{EntityUID: unshareWith}]; ok {
		s.logger.InfoContext(
			r.Context(),
			"found unshare_with User entity",
			slog.Any("unshareWithUser", unshareWithUser),
		)
		unshareWithUser.Parents = removeParent(unshareWithUser.Parents, roleTeamUID.EntityUID)
		s.logger.InfoContext(
			r.Context(),
			"remove parent from unshare_with User entity",
			slog.Any("unshareWithUser", unshareWithUser),
			slog.Any("removed", roleTeamUID),
		)
	} else {
		s.logger.InfoContext(
			r.Context(),
			"failed to find unshare_with entity",
			slog.String("unshare_with", req.UnshareWith),
		)
		s.writeGenericResponse(
			r.Context(),
			w,
			&GenericResponse{
				Error: "Failed to find unshare_with entity",
			},
			http.StatusBadRequest,
		)
		return
	}

	s.writeGenericResponse(
		r.Context(),
		w,
		&GenericResponse{
			Message: "ok", // this particular message is required by tinytodo.py
		},
		http.StatusOK,
	)
	return
}
