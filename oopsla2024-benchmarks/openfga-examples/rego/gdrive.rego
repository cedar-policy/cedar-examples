package gdrive
import future.keywords.in

################################################################################
#### GDrive Rego Policy
#### input -> dict:
####   principal -> dict:
####   	documentAndFolderWithViewAccess : list of string
####   	ownedFolders : list of string
####   	ownedDocuments : list of string
####   action -> string
####   resource -> dict:
####    uid : string
####	isPublic : bool
####   files : file-graph
################################################################################

transitive_actions := ["Action::\"read\"", "Action::\"write\"", "Action::\"share\""]

## Any public resource can be read
allow {
	input.action == "Action::\"read\""
	input.resource.isPublic
}

## Any file reachable from our view-set can be read
allow {
	input.action == "Action::\"read\""
	input.resource.uid in graph.reachable(input.files, input.principal.documentsAndFoldersWithViewAccess)
}

# Any file we own can be read/write/share-d
allow {
	input.action in transitive_actions
	input.resource.uid in input.principal.ownedDocuments
}

# Any file contains in a folder we own can be read/write/share-d
allow {
	input.action in transitive_actions
	input.principal.ownedFolders[_] in graph.reachable(input.files, [input.resource.uid])

}

# Any document we own we can change the owner of
allow {
	input.action == "Action::\"changeOwner\""
	input.resource.uid in input.principal.ownedDocuments
}

# Any folder we own we can create a document in
allow {
	input.action == "Action::\"createDocument\""
	input.resource.uid in input.principal.ownedFolders
}


