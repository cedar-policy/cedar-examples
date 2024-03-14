#!/usr/bin/env python3

import sys
import random
import json

def euid(type, id):
    """type and id should be strings. returns json object as Python map"""
    return { "__entity": { "type": type, "id": id }}

def entity(uid, attrs, parents):
    """uid should be a json object produced by euid(), attrs a map, and parents a list. returns json object as Python map"""
    return { "uid": uid, "attrs": attrs, "parents": parents }

def action_entities():
    """returns list of (json object as Python map)"""
    return [
        entity(euid("Action", "CreateList"), attrs={}, parents=[]),
        entity(euid("Action", "GetList"), attrs={}, parents=[]),
        entity(euid("Action", "UpdateList"), attrs={}, parents=[]),
        entity(euid("Action", "DeleteList"), attrs={}, parents=[]),
        entity(euid("Action", "GetLists"), attrs={}, parents=[]),
        entity(euid("Action", "CreateTask"), attrs={}, parents=[]),
        entity(euid("Action", "UpdateTask"), attrs={}, parents=[]),
        entity(euid("Action", "DeleteTask"), attrs={}, parents=[]),
        entity(euid("Action", "EditShares"), attrs={}, parents=[]),
    ]

def app_euid():
    return euid("Application", "TinyTodo")

def app_entity():
    """returns json object as Python map"""
    return entity(app_euid(), attrs={}, parents=[])

def task():
    """returns a random task, as a json object as Python map"""
    return { "name": "task_"+str(random.randint(1,1000000)), "id": random.randint(1,1000000), "state": "Checked" if random.randint(1,2)==1 else "Unchecked" }

def list_entity(name, owner, readers, editors):
    """name should be string, owner a User euid, readers/editors a Team euid. returns json object as Python map"""
    return entity(
        euid("List", name),
        attrs={
            "name": name,
            "owner": owner,
            "readers": readers,
            "editors": editors,
            "tasks": [task() for _ in range(random.randint(0,50))],
        },
        parents=[app_euid()]
    )

def user_entity(name, team_parents):
    """name should be string, team_parents should be list of Team euids. returns json object as Python map"""
    return entity(
        euid("User", name),
        attrs={},
        parents=team_parents + [app_euid()]
    )

def team_entity(name, team_parents):
    """name should be string, team_parents should be list of Team euids. returns json object as Python map"""
    return entity(euid("Team", name), attrs={}, parents=team_parents + [app_euid()])

# out_file: filename of the output file, or `None` to write to stdout
def build_entities(num_users, num_teams, num_lists, connection_probability, out_file=None):
    chance_user_in_team = connection_probability
    chance_team_in_team = connection_probability

    entities = [app_entity()] # list of JSON objects (as Python maps)
    #entities.extend(action_entities()) # commented out -- calling code extracts these from schema
    for i in range(num_users):
        user_name = "user_"+str(i)
        parents = []
        for j in range(num_teams):
            if random.random() < chance_user_in_team:
                parents.append(euid("Team", "team_"+str(j)))
        entities.append(user_entity(user_name, parents))
    for i in range(num_teams):
        team_name = "team_"+str(i)
        parents = []
        for j in range(i): # teams only in teams with smaller indexes, ensures DAG
            if random.random() < chance_team_in_team:
                parents.append(euid("Team", "team_"+str(j)))
        entities.append(team_entity(team_name, parents))
    for i in range(num_lists):
        list_name = "list_"+str(i)
        owner = euid("User", "user_"+str(random.randint(0,num_users-1)))
        readers = euid("Team", "team_"+str(random.randint(0,num_teams-1)))
        editors = euid("Team", "team_"+str(random.randint(0,num_teams-1)))
        entities.append(list_entity(list_name, owner, readers, editors))

    handle = open(out_file, "w") if out_file is not None else sys.stdout
    json.dump(entities, handle, indent=2)
    handle.flush()
    if handle is not sys.stdout:
        handle.close()


def print_usage():
    print("Usage: tinytodo_entity_generator.py num_users num_teams num_lists connection_probability [output_file_name]", file=sys.stderr)

def main():
    if len(sys.argv) < 5:
        print("error: got " + str(len(sys.argv)-1) + " parameters, expected at least 4", file=sys.stderr)
        print_usage()
        sys.exit(1)
    if len(sys.argv) > 6:
        print("error: got " + str(len(sys.argv)-1) + " parameters, expected at most 5", file=sys.stderr)
        print_usage()
        sys.exit(1)

    num_users = int(sys.argv[1])
    num_teams = int(sys.argv[2])
    num_lists = int(sys.argv[3])
    connection_probability = float(sys.argv[4])
    output_file_name = sys.argv[5] if len(sys.argv) >= 6 else None

    build_entities(num_users, num_teams, num_lists, connection_probability, out_file=output_file_name)

if __name__ == "__main__":
    main()
