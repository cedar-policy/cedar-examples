#!/usr/bin/env python3

import sys
import random
import json

def euid(type, id):
    """type and id should be strings. returns json object as Python map"""
    return { "__entity": { "type": type, "id": id }}

def entity(uid, attrs, parents):
    """uid should be a string, attrs a map, and parents a list. returns json object as Python map"""
    return { "uid": uid, "attrs": attrs, "parents": parents }

def action_entities():
    """returns list of (json object as Python map)"""
    return [
        entity(euid("Action", "createDocument"), attrs={}, parents=[]),
        entity(euid("Action", "changeOwner"), attrs={}, parents=[]),
        entity(euid("Action", "share"), attrs={}, parents=[]),
        entity(euid("Action", "write"), attrs={}, parents=[]),
        entity(euid("Action", "read"), attrs={}, parents=[]),
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

def folder_entity(name, parents):
    """name should be string, parents should be list. returns json object as Python map"""
    return entity(euid("Folder", name), attrs={}, parents=parents)

def doc_entity(name, parents):
    """name should be string, parents should be list. returns json object as Python map"""
    return entity(euid("Document", name), attrs={ "isPublic": True if random.randint(1, 2) == 1 else False }, parents=parents)

def user_entities(name, parents, owned_docs, owned_folders):
    """name should be string, parents/owned_docs/owned_folders should be lists. returns list of (json object as Python map)"""
    return [
        entity(
            euid("User", name),
            attrs={
                "ownedDocuments": owned_docs,
                "ownedFolders": owned_folders,
            },
            parents=parents
        ),
    ]

def group_entities(name, user_names):
    """name should be string, and user_names should be a list of users in the group (as eid strings). returns list of (json object as Python map)"""
    return [
        entity(euid("Group", name), attrs={}, parents=[]),
    ]

# out_file: filename of the output file, or `None` to write to stdout
def build_entities(num_users, num_groups, num_docs, num_folders, connection_probability, out_file=None):
    chance_user_in_group = connection_probability
    chance_user_owns_doc = connection_probability
    chance_user_owns_folder = connection_probability
    chance_doc_in_folder = connection_probability
    chance_folder_in_folder = connection_probability
    chance_user_view_doc = connection_probability
    chance_group_view_doc = connection_probability
    chance_user_view_folder = connection_probability
    chance_group_view_folder = connection_probability

    entities = [] # list of JSON objects (as Python maps)
    #entities.extend(action_entities()) # commented out -- calling code extracts these from schema
    group_membership = { i : [] for i in range(num_groups) } # maps group number to a list of user numbers in that group
    for i in range(num_users):
        user_name = "user_"+str(i)
        parents = []
        for j in range(num_groups):
            if random.random() < chance_user_in_group:
                parents.append(euid("Group", "group_"+str(j)))
                group_membership[j] = group_membership[j] + [i]
        owned_docs = []
        for j in range(num_docs):
            if random.random() < chance_user_owns_doc:
                owned_docs.append(euid("Document", "doc_"+str(j)))
        owned_folders = []
        for j in range(num_folders):
            if random.random() < chance_user_owns_folder:
                owned_folders.append(euid("Folder", "folder_"+str(j)))
        entities.extend(user_entities(user_name, parents, owned_docs, owned_folders))
    for i in range(num_groups):
        group_name = "group_"+str(i)
        entities.extend(group_entities(group_name, ["user_"+str(j) for j in group_membership[i]]))
    for i in range(num_docs):
        doc_name = "doc_"+str(i)
        parents = []
        for j in range(num_folders):
            if random.random() < chance_doc_in_folder:
                parents.append(euid("Folder", "folder_"+str(j)))
        for j in range(num_users):
            if random.random() < chance_user_view_doc:
                entities.append(template_link("template", euid("User", "user_"+str(j)), euid("Document", doc_name)))
        for j in range(num_groups):
            if random.random() < chance_group_view_doc:
                entities.append(template_link("template", euid("Group", "group_"+str(j)), euid("Document", doc_name)))
        entities.append(doc_entity(doc_name, parents))
    for i in range(num_folders):
        folder_name = "folder_"+str(i)
        parents = []
        for j in range(i): # folders only in folders with smaller indexes, ensures DAG
            if random.random() < chance_folder_in_folder:
                parents.append(euid("Folder", "folder_"+str(j)))
        for j in range(num_users):
            if random.random() < chance_user_view_folder:
                entities.append(template_link("template", euid("User", "user_"+str(j)), euid("Folder", folder_name)))
        for j in range(num_groups):
            if random.random() < chance_group_view_folder:
                entities.append(template_link("template", euid("Group", "group_"+str(j)), euid("Folder", folder_name)))
        entities.append(folder_entity(folder_name, parents))

    handle = open(out_file, "w") if out_file is not None else sys.stdout
    json.dump(entities, handle, indent=2)
    handle.flush()
    if handle is not sys.stdout:
        handle.close()


def print_usage():
    print("Usage: gdrive_templates_entity_generator.py num_users num_groups num_docs num_folders connection_probability [output_file_name]", file=sys.stderr)

def main():
    if len(sys.argv) < 6:
        print("error: got " + str(len(sys.argv)-1) + " parameters, expected at least 5", file=sys.stderr)
        print_usage()
        sys.exit(1)
    if len(sys.argv) > 7:
        print("error: got " + str(len(sys.argv)-1) + " parameters, expected at most 6", file=sys.stderr)
        print_usage()
        sys.exit(1)

    num_users = int(sys.argv[1])
    num_groups = int(sys.argv[2])
    num_docs = int(sys.argv[3])
    num_folders = int(sys.argv[4])
    connection_probability = float(sys.argv[5])
    output_file_name = sys.argv[6] if len(sys.argv) >= 7 else None

    build_entities(num_users, num_groups, num_docs, num_folders, connection_probability, out_file=output_file_name)

if __name__ == "__main__":
    main()
