import sys
import random


def write_repo(name):
    s = '''
    {
        "uid": { "__entity": { "type": "Repository", "id": "NAME"} },
        "attrs": {
            "readers" : { "__entity": { "type": "UserGroup", "id": "NAME_readers"} },
            "triagers" : { "__entity": { "type": "UserGroup", "id": "NAME_triagers"} },
            "writers" : { "__entity": { "type": "UserGroup", "id": "NAME_writers"} },
            "maintainers" : { "__entity": { "type": "UserGroup", "id": "NAME_maintainers"} },
            "admins" : { "__entity": { "type": "UserGroup", "id": "NAME_admins"} }
        },
        "parents": []
    },
    {
        "uid": { "__entity": { "type": "UserGroup", "id": "NAME_readers"} },
        "attrs": {
        },
        "parents": [  ]
    },
    {
        "uid": { "__entity": { "type": "UserGroup", "id": "NAME_triagers"} },
        "attrs": {
        },
        "parents": [ { "__entity": { "type": "UserGroup", "id": "NAME_readers"} } ]
    },
    {
        "uid": { "__entity": { "type": "UserGroup", "id": "NAME_writers"} },
        "attrs": {
        },
        "parents": [ {"__entity": { "type": "UserGroup", "id": "NAME_triagers"}} ]
    },
    {
        "uid": { "__entity": { "type": "UserGroup", "id": "NAME_maintainers"} },
        "attrs": {
        },
        "parents": [ {"__entity": { "type": "UserGroup", "id": "NAME_writers"}} ]
    },
    {
        "uid": { "__entity": { "type": "UserGroup", "id": "NAME_admins"} },
        "attrs": {
        },
        "parents": [ {"__entity": { "type": "UserGroup", "id": "NAME_maintainers"}} ]
    }'''
    s=s.replace("NAME", name)
    return s


def write_user(name, parent_string):
    s = '''
    {
      "uid": { "__entity": { "type": "User", "id": "NAME"} },
      "attrs": {},
      "parents": PARENT_STRING
    }'''
    s=s.replace("NAME", name)
    s=s.replace("PARENT_STRING", parent_string)
    return s

def write_team(name, parent_string, depth):
    i = 0
    s = '''
    {
      "uid": { "__entity": { "type": "Team", "id": "NAME"} },
      "attrs": {},
      "parents": PARENT_STRING
    },'''
    s=s.replace("NAME", name+"_"+str(i))
    s=s.replace("PARENT_STRING", parent_string)
    i = i+1
    while i < depth:
        s_prime = '''
    {
    "uid": { "__entity": { "type": "Team", "id": "NAME"} },
    "attrs": {},
    "parents": [{"__entity": {"type": "Team", "id": "TEAM_I_MINUS_1"}}]
    },'''
        s_prime=s_prime.replace("NAME", name+"_"+str(i))
        s_prime=s_prime.replace("TEAM_I_MINUS_1", name+"_"+str(i-1))
        s = s+s_prime
        i = i + 1
    return s[:-1]


def build_parent_string(strings):
    s = '''{ "__entity": { "type": "UserGroup", "id": "GROUP_NAME"} }'''
    r = "["
    for group_name in strings:
        if group_name.find("team_") != -1:
            r = r+s.replace("UserGroup","Team").replace("GROUP_NAME",group_name)+","
        else:
            r = r+s.replace("GROUP_NAME",group_name)+","
    if len(strings) > 0:
        r = r[:-1]
    r = r+"]"
    return r

def random_role_str():
    p = random.random()
    if p < 0.2:
        return "_readers"
    if p < 0.4:
        return "_triagers"
    if p < 0.6:
        return "_writers"
    if p < 0.8:
        return "_maintainers"
    else:
        return "_admins"

def build_entities(out_file):
    num_repos = 1000
    num_users = 1000
    num_teams = 100
    chance_user_use_repo = 0.05
    chance_team_use_repo = 0.05
    chance_user_in_team = 0.05

    # For benchmarking, we'll nest teams to this depth
    team_stack_height = 1

    with open(out_file, "w") as file:
        file.write("[")
        for i in range(num_repos):
            repo_name = "repo_"+str(i)
            s = write_repo(repo_name)
            file.write(s+",")
        for i in range(num_teams):
            team_name = "team_"+str(i)
            parents = []
            for j in range(num_repos):
                if random.random() < chance_team_use_repo:
                    parents.append("repo_"+str(j)+random_role_str())
            parent_s = build_parent_string(parents)
            s = write_team(team_name, parent_s, team_stack_height)
            file.write(s+",")
        for i in range(num_users):
            user_name = "user_"+str(i)
            parents = []
            for j in range(num_repos):
                if random.random() < chance_user_use_repo:
                    parents.append("repo_"+str(j)+random_role_str())
            for j in range(num_teams):
                if random.random() < chance_user_in_team:
                    parents.append("team_"+str(j)+"_"+str(team_stack_height-1))
            parent_s = build_parent_string(parents)
            s = write_user(user_name, parent_s)
            if i < num_users-1:
                file.write(s+",") 
            else:
                file.write(s)
        file.write("]")


def main():
    if len(sys.argv) != 2:
        print("Usage: generate_entities.py output_file_name")
    else:
        build_entities(sys.argv[1])

if __name__ == "__main__":
    main()
