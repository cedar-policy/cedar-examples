#!/usr/bin/env python3

import sys
import random
import json

def euid(type, id):
    """type and id should be strings. returns json object as Python map"""
    return { "__entity": { "type": type, "id": id }}

def entity(uid, attrs, parents):
    """uid should be a json object as Python map, attrs a map, and parents a list. returns json object as Python map"""
    return { "uid": uid, "attrs": attrs, "parents": parents }

def action_entities():
    """returns list of (json object as Python map)"""
    return [
        entity(euid("Action", "read"), attrs={}, parents=[euid("Action", "triage")]),
        entity(euid("Action", "triage"), attrs={}, parents=[euid("Action", "write")]),
        entity(euid("Action", "write"), attrs={}, parents=[euid("Action", "maintain")]),
        entity(euid("Action", "maintain"), attrs={}, parents=[euid("Action", "admin")]),
        entity(euid("Action", "admin"), attrs={}, parents=[]),
    ]

def template_link(template_name, principal, resource):
    """
    currently, the CedarBenchmarks code expects a python entity generator to
    produce entity JSON on stdout, which doesn't provide an easy way to also
    generate template-linked policies.
    As a hack, we generate special (fictitious) entities of type TemplateLink,
    which the CedarBenchmarks code will process and remove.

    template_name should be a string, principal and resource should be euid
    objects (as Python map)
    """
    return entity(
        euid("TemplateLink", template_name + "_" + principal["__entity"]["id"] + "_" + resource["__entity"]["id"]),
        attrs={
            "template_name": template_name,
            "principal": principal,
            "resource": resource,
        },
        parents=[]
    )

def org_entities(name, parents):
    """returns list of (json object as Python map)"""
    return [
        entity(
            euid("Organization", name),
            attrs={
                "readers": euid("OrgPermission", name+"_readers"),
                "writers": euid("OrgPermission", name+"_writers"),
                "admins": euid("OrgPermission", name+"_admins"),
            },
            parents=parents
        ),
        entity(euid("OrgPermission", name+"_readers"), attrs={}, parents=[]),
        entity(euid("OrgPermission", name+"_writers"), attrs={}, parents=[]),
        entity(euid("OrgPermission", name+"_admins"), attrs={}, parents=[]),
    ]

def repo_entities(name, owner):
    """name and owner should be string. returns list of (json object as Python map)"""
    return [
        entity(
            euid("Repository", name),
            attrs={
                "owner": euid("Organization", owner)
            },
            parents = []
        ),
    ]

def user_entity(name, parents):
    """name should be string, parents should be list. returns json object as Python map"""
    return entity(euid("User", name), attrs={}, parents=parents)

def team_entity(name, parents):
    """name should be string, parents should be list. returns json object as Python map"""
    return entity(euid("Team", name), attrs={}, parents=parents)

def random_template_name():
    p = random.random()
    if p < 0.2:
        return "readTemplate"
    if p < 0.4:
        return "triageTemplate"
    if p < 0.6:
        return "writeTemplate"
    if p < 0.8:
        return "maintainTemplate"
    else:
        return "adminTemplate"

def random_org_role_str():
    p = random.randint(1, 3)
    if p == 1:
        return "_readers"
    if p == 2:
        return "_writers"
    else:
        return "_admins"

# out_file: filename of the output file, or `None` to write to stdout
def build_entities(num_repos, num_users, num_teams, num_orgs, connection_probability, out_file=None):
    chance_org_use_org = connection_probability
    chance_team_in_team = connection_probability
    chance_team_use_repo = connection_probability
    chance_user_use_repo = connection_probability
    chance_user_in_team = connection_probability
    chance_user_in_org = connection_probability
    chance_user_use_org = connection_probability

    entities = [] # list of JSON objects (as Python maps)
    #entities.extend(action_entities()) # commented out -- calling code extracts these from schema
    for i in range(num_orgs):
        org_name = "org_"+str(i)
        parents = []
        for j in range(num_orgs):
            if random.random() < chance_org_use_org:
                parents.append(euid("OrgPermission", "org_"+str(j)+random_org_role_str()))
        entities.extend(org_entities(org_name, parents))
    for i in range(num_repos):
        repo_name = "repo_"+str(i)
        owner = "org_"+str(random.randint(0, num_orgs-1))
        entities.extend(repo_entities(repo_name, owner))
    for i in range(num_teams):
        team_name = "team_"+str(i)
        parents = []
        for j in range(i): # teams only in teams with smaller indexes, ensures DAG
            if random.random() < chance_team_in_team:
                parents.append(euid("Team", "team_"+str(j)))
        for j in range(num_repos):
            if random.random() < chance_team_use_repo:
                entities.append(template_link(random_template_name(), euid("Team", team_name), euid("Repository", "repo_"+str(j))))
        entities.append(team_entity(team_name, parents))
    for i in range(num_users):
        user_name = "user_"+str(i)
        parents = []
        for j in range(num_repos):
            if random.random() < chance_user_use_repo:
                entities.append(template_link(random_template_name(), euid("User", user_name), euid("Repository", "repo_"+str(j))))
        for j in range(num_teams):
            if random.random() < chance_user_in_team:
                parents.append(euid("Team", "team_"+str(j)))
        for j in range(num_orgs):
            if random.random() < chance_user_in_org:
                parents.append(euid("Organization", "org_"+str(j)))
            if random.random() < chance_user_use_org:
                parents.append(euid("OrgPermission", "org_"+str(j)+random_org_role_str()))
        entities.append(user_entity(user_name, parents))

    handle = open(out_file, "w") if out_file is not None else sys.stdout
    json.dump(entities, handle, indent=2)
    handle.flush()
    if handle is not sys.stdout:
        handle.close()


def print_usage():
    print("Usage: github_templates_entity_generator.py num_repos num_users num_teams num_orgs connection_probability [output_file_name]", file=sys.stderr)

def main():
    if len(sys.argv) < 6:
        print("error: got " + str(len(sys.argv)-1) + " parameters, expected at least 5", file=sys.stderr)
        print_usage()
        sys.exit(1)
    if len(sys.argv) > 7:
        print("error: got " + str(len(sys.argv)-1) + " parameters, expected at most 6", file=sys.stderr)
        print_usage()
        sys.exit(1)

    num_repos = int(sys.argv[1])
    num_users = int(sys.argv[2])
    num_teams = int(sys.argv[3])
    num_orgs = int(sys.argv[4])
    connection_probability = float(sys.argv[5])
    output_file_name = sys.argv[6] if len(sys.argv) >= 7 else None

    build_entities(num_repos, num_users, num_teams, num_orgs, connection_probability, out_file=output_file_name)

if __name__ == "__main__":
    main()
