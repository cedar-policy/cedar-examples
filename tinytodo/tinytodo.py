#  Copyright 2022-2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
#
#  Licensed under the Apache License, Version 2.0 (the "License");
#  you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
#
#       https://www.apache.org/licenses/LICENSE-2.0
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.


import requests
import json
import os
import subprocess
import atexit


# Classes for managing Cedar Entities
class Entity:

    def __repr__(self):
        return self.euid()

    def __str__(self):
        return self.name

    def euid(self):
        return '%s::"%s"' % (self.type, self.name)
    
    def eid(self):
        return self.name

class User(Entity):

    def __init__(self, name):
        self.type = 'User'
        self.name = name

class Team(Entity):
    def __init__(self, name):
        self.type = 'Team'
        self.name = name


class List(Entity):

    def __init__(self, x):
        self.type = 'List'
        if type(x) is str:
            self.name = parse_euid(x, self.type)
        else:
            self.name = str(x)



def parse_euid(euid, expected_type):
    parts = euid.split('::')
    if len(parts) == 2:
        typ = parts[0]
        name = parts[1][1:-1]
        if typ == expected_type:
            return name
        else:
            raise Exception('Expected type: %s, got %s' % (expected_type, typ))
    else:
        raise Exception('Failed to parse euid: {}' % euid)


def attempt_build():
    print('Attempting to build the tiny-todo-server via Cargo')
    proc = subprocess.run(['cargo', 'build', '--release'], check = True)
    print('Build successfull')


# Class for managing a tinytodo server process
class Server:
    def __init__(self, port):
        self.port = port
        if not os.path.isfile(server_binary_path):
            try:
                attempt_build()
            except:
                print('Unable to build using cargo!')
                return
        self.proc =  subprocess.Popen([server_binary_path, port])
        print('TinyTodo server started on port %s' % port)


    def kill(self):
        self.proc.terminate()
        print('TinyTodo server stopped on port %s' % self.port)


    def url(self):
        return 'http://localhost:%s' % self.port

    def get(self, param):
        return requests.get('%s%s' % (self.url(), param))

    def post(self, param, data):
        return requests.post('%s%s' % (self.url(), param), json = data)

    def delete(self, param, data):
        return requests.delete('%s%s' % (self.url(), param), json = data)

    def stopped(self):
        return False

class StoppedServer:

    def kill(self):
        pass

    def stopped(self):
        return True
    
server = StoppedServer()
server_binary_path = './target/release/tiny-todo-server'


# Setup our entities
users = ['emina', 'aaron', 'andrew', 'kesha']
[emina, aaron, andrew, kesha] = [User(user) for user in users]
teams = ['admin', 'interns', 'temp']
[admin, interns, temp] = [Team(team) for team in teams]
current_user = None







# Set the current user
def set_user(user):
    global current_user
    current_user = user
    print('User is now %s' % user)

# Start the TinyTodo server
def start_server(port = 8080):
    global server
    if server.stopped():
        server = Server(str(port))
    else:
        print('Server is already running')


# Stop the TinyTodo server
def stop_server():
    global server
    if not server.stopped():
        server.kill()
        server = StoppedServer()

# Ensure the server gets stop at process exit
atexit.register(stop_server)

class AuthException(Exception):
    def __init__(self, resp):
        self.resp = resp

class NoSuchTaskException(Exception):
    def __init__(self, lst, task_id):
        self.lst = lst
        self.task_id = task_id

    def __str__(self):
        return 'No such task id %d on list %s' % (self.task_id, self.lst)


# Decorator for API-calling functions.
# Each function has a name, and returns a tuple (resp, f)
# where f is a function taking the APIs successful response body
# and pretty-printing.
# The decorator handles error processing
def web_req(name):
    def decorator(func):
        def wrapper(*args, **kwargs):
            global current_user, server
            if server.stopped():
                print('No server running! Use `start_server()`!')
                return
            if current_user is None:
                print('No user set! Use `set_user()`')
                return
            try:
                resp,f = func(current_user, *args, **kwargs)
                process_response(name, resp, f, args)
            except AuthException as e:
                process_response(name, e.resp, lambda x : 'Unreachable', args)
            except NoSuchTaskException as e:
                print(e)
        return wrapper
    return decorator

def process_response(name, resp, f, args):
    if resp.status_code == 200:
        body = json.loads(resp.text)
        if is_error(body):
            if is_authz_denied(body):
                args = ','.join(map(str, args))
                tup = (current_user,  name, args)
                print('Access denied. User %s is not authorized to %s on [%s]' % tup )
            else:
                print('Error: %s' % body['error'])
        else:
            print(f(body))
    else:
        print('HTTP Error. Status: %d, body: %s' % (resp.status_code, resp.text))
            


def is_error(body):
    return type(body) is dict and 'error' in body


def is_authz_denied(body):
    return 'error' in body and body['error'] == 'Authorization Denied'
        


### API Calling Functions ###


@web_req("Get Lists")
def get_lists(user):
    req = server.get('/api/lists/get?uid=%s' % user.eid())
    return req, get_lists_printer(user)

def get_lists_printer(user):
    def inner(list_of_lists):
        if len(list_of_lists) == 0:
            return 'No lists for %s' % user
        else:
            return 'Lists: %s' % ','.join([lst['name'] for lst in sorted(list_of_lists, key=lambda l: int(parse_euid(l['uid'], 'List')))])

    return inner

@web_req("Create List")
def create_list(user, name):
    data = {
            'uid' : user.eid(),
            'name' : name
            }
    f = lambda x: 'Created list ID %s' % List(x)
    return server.post('/api/list/create', data), f

@web_req("Get List")
def get_list(user, list_id):
    l = List(list_id)
    return get_list_inner(user, l), display_list(l)

def get_list_inner(user, lst):
    return server.get('/api/list/get?uid=%s&list=%s' % (user.eid(), lst.eid()))

def get_list_data(user, lst):
    resp = get_list_inner(user, lst)
    body = json.loads(resp.text)
    if 'error' in body:
        raise AuthException(resp)
    else:
        return body


def display_list(l):
    def inner(obj):
        title = '=== %s ===' % obj['name']
        id_line = 'List ID: %s' % l
        owner_line = 'Owner: %s' % obj['owner']
        tasks_header = 'Tasks:'
        list_of_tasks = obj['tasks']
        list_of_tasks.sort(key = lambda task: task['id'])
        lines = [title, id_line, owner_line, tasks_header] + [display_task(i + 1, task) for (i, task) in enumerate(list_of_tasks)]
        return '\n'.join(lines)
    return inner


def display_task(index, task):
    return '%d: %s %s' % (index, '[ ]' if task['state'] == 'Unchecked' else '[X]', task['name'])


            

@web_req("Create Task")
def create_task(user, list_id, name):
    url = '/api/task/create'
    data = { 
            'uid' : user.eid(),
            'list' : List(list_id).eid(),
            'name' : name
            }
    return server.post(url, data), lambda _ : 'Created task on list ID %d' % list_id



@web_req("Toggle Task")
def toggle_task(user, list_id, task_id):
    lst = List(list_id)
    task = find_task(user, lst, task_id)
    url = '/api/task/update'
    data = {
            'uid' : user.eid(), 
            'list' : lst.eid(), 
            'task' : task['id'],
            'state' : toggle_state(task['state'])
            }
    return server.post(url, data), lambda _: 'Toggled task on list ID %s' % lst

def find_task(user, lst, task_id):
    task_id = task_id - 1
    current_list = get_list_data(user, lst)
    list_of_tasks = current_list['tasks']
    list_of_tasks.sort(key = lambda task : task['id'])
    if task_id < len(list_of_tasks):
        return list_of_tasks[task_id]
    else:
        raise NoSuchTaskException(lst, task_id + 1)

@web_req("Change Task Description")
def change_task_description(user, list_id, task_id, desc):
    lst = List(list_id)
    task = find_task(user, lst, task_id)
    url = '/api/task/update'
    data = { 
            'uid' : user.eid(), 
            'list' : lst.eid(),
            'task' : task['id'], 
            'name' : desc
            }
    return server.post(url, data), lambda _: 'Description Updated'

@web_req("Delete Task")
def delete_task(user, lst_id, task_id):
    lst = List(lst_id)
    task = find_task(user, lst, task_id)
    url = '/api/task/delete'
    data = {
            'uid' : user.eid(),
            'list' : lst.eid(), 
            'task' : task['id'],
            }
    return server.delete(url, data), lambda _: 'Task Deleted'

@web_req("delete list")
def delete_list(user, list_id):
    url = '/api/list/delete'
    data = {
            'uid' : user.eid(),
            'list' : List(list_id).eid(),
            }
    return server.delete(url, data), lambda _: 'List Deleted'


@web_req("share list")
def share_list(user, list_id, share_with, read_only = True):
    l = List(list_id)
    url = '/api/share'
    data = {
            'uid' : user.eid(), 
            'list' : l.eid(), 
            'role' : 'Reader' if read_only else 'Editor',
            'share_with' : share_with.euid(),
            }
    return server.post(url, data), lambda _: 'Shared list ID %s with %s as %s' % (l, share_with, 'reader' if read_only else 'editor')

@web_req("unshare list")
def unshare_list(user, list_id, unshare_with, read_only = True):
    l = List(list_id)
    url = '/api/share'
    data = {
            'uid' : user.eid(), 
            'list' : l.eid(), 
            'role' : 'Reader' if read_only else 'Editor',
            'unshare_with' : unshare_with.euid(),
            }
    return server.delete(url, data), lambda _: 'Unshared %s permissions on list ID %s with %s' % ('read' if read_only else 'edit', l, unshare_with)



def toggle_state(s):
    if s == 'Unchecked':
        return 'Checked'
    elif s == 'Checked':
        return 'Unchecked'
    else:
        raise Error('Invalid state: %s' % s)
    


