# TinyTodo

TinyTodo is a simple application for managing task lists. It uses Cedar to control who has access to what.

TinyTodo allows individuals, called `Users` to organize, track, and share their todo lists. `Users` create `Lists` which they can populate with tasks. As tasks are completed, they can be checked off the list. A list's creater, called its _owner_, can share a list with either `User`s or `Team`s, either as a _reader_ or an _editor_. A reader can only view the contents of a list, while an editor can modify it (e.g., add tasks, or check them off the list).

## Usage

The code is structured as a server, written in Rust, that processes HTTP commands. A client `tinytodo.py`, written in Python3, can be used to interact with the server. This is just a demo app, so there is no permanent storage of todo lists -- they last only as long as the server is running.

### Build

You need Python3 and Rust. Rust can be installed via [rustup](https://rustup.rs). Python3 can be installed [here](https://www.python.org/) or using your system's package manager.

This example expects that the [`cedar`](https://github.com/cedar-policy/cedar) repository is cloned into the toplevel (`../cedar-examples`) directory. You can instruct Cargo to use your local version of `cedar-policy` by adding `path = "../cedar/cedar-policy"` to Cargo.toml.

Install the needed python packages in a virtual environment, and build the server as follows.

```shell
python3 -m venv ./venv
source ./venv/bin/activate
pip3 install -r requirements.txt
cargo build --release
```

The Rust executable is stored in `target/release/tiny-todo-server`.

### Run

To start the client within Python interactive mode, enter

```shell
python3 -i ./tinytodo.py
```

To start the server, from the Python primary prompt `>>>` enter

```python
start_server()
```

When it starts up, the server reads in the Cedar policies in `policies.cedar`, and the Cedar entities, which define the TinyTodo `User`s and `Team`s, from `entities.json`. It validates the policies are consistent with `tinytodo.cedarschema.json`, and will abort if they are not.

Client code `tinytodo.py` defines the functions you can call, which serve as the list of commands. See also [`TUTORIAL.md`](./TUTORIAL.md) for a detailed description of how to use these commands, and how TinyTodo works. Here is a brief description of the commands:

* `start_server()` -- starts the TinyTodo server on port 8080. To use port XXX instead, provide `port=XXX` as the argument instead. Fails if server is already running.
* `stop_server()` -- shuts down the TinyTodo server, if running. Called automatically on exit.
* `set_user(user)` -- sets the user to use for the commands that follow. Parameter `user` can be any of `emina`, `aaron`, `andrew`, or `kesha`. In the following commands, you can override this user by providing an additional parameter at the start that names the user.
* `get_lists()` -- gives the lists owned by the current user
* `create_list(name)` -- creates the list named `name` (a string) owned by the current user; prints the numeric ID of the created list on success
* `get_list(list)` -- gets information about list `list`, indicated by its numeric ID.
* `create_task(list,name)` -- creates a new (uncompleted) task for list `list` named `name` (a string); prints the task's numeric ID on success, which is its position in the list
* `toggle_task(list,task)` -- toggles the completion status of the task `task` (a numeric ID) for list `list`
* `change_task_description(list,task,name)` -- changes the name of task `task` in list `list` to `name` (a string)
* `delete_task(list,task)` -- deletes task `task` from list `list`. Reorders remaining tasks
* `delete_list(list)` -- deletes the given list
* `share_list(list,target,readonly)` -- shares the given list with `target`; if `readonly` (a boolean) is `True` then the target has _reader_ status for the list, else _editor_ status. `readonly` is an optional parameter, defaulting to readonly. `target` can be a user or a team, where legal teams are `temp`, `interns`, and `admin`
* `unshare_list(list,target)` -- revokes access to `list` for `target`, which can be a user or a team
