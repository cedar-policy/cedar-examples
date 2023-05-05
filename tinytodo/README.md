# TinyTodo

TinyTodo is a simple application for managing task lists. It uses Cedar to control who has access to what.

TinyTodo allows individuals, called `Users` to organize, track, and share their todo lists. `Users` create `Lists` which they can populate with tasks. As tasks are completed, they can be checked off the list. A list's creater, called its _owner_, can share a list a list with either `User`s or `Team`s, either as a _reader_ or an _editor_. A reader can only view the contents of a list, while an editor can modify it (e.g., add tasks, or check them off the list).

## Usage

The code is structured as a server, written in Rust, that processes HTTP commands. A client `tinytodo.py`, written in Python3, can be used to interact with the server. This is just a demo app, so there is no permanent storage of todo lists -- they last only as long as the server is running.

### Build

You need Python3 and Rust. Install the needed python packages, and build the server as follows. 
```shell
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

Look at the `tinytodo.py` code to see the functions you can call, which serve as the list of commands. See also [`TUTORIAL.md`](./TUTORIAL.md) for a detailed description of how to use these commands, and how TinyTodo works.
