// allow resource.owner full access to Photos
permit (
  principal,
  action in
    [PhotoApp::Action::"viewPhoto",
     PhotoApp::Action::"editPhoto",
     PhotoApp::Action::"deletePhoto"],
  resource in PhotoApp::Application::"PhotoApp"
)
when { resource.owner == principal };

// deny access to private Photos from non-owner
forbid (
  principal,
  action,
  resource in PhotoApp::Application::"PhotoApp"
)
when { resource.labels.contains("private") }
unless { resource.owner == principal };

// allow entity subjects to view Photos
permit (
  principal,
  action == PhotoApp::Action::"viewPhoto",
  resource in PhotoApp::Application::"PhotoApp"
)
when { resource has subjects && resource.subjects.contains(principal) };

// conditionally allow PhotoJudge to view "contest" Photos
permit (
  principal in PhotoApp::Role::"PhotoJudge",
  action == PhotoApp::Action::"viewPhoto",
  resource in PhotoApp::Application::"PhotoApp"
)
when { resource.labels.contains("contest") }
when { context has judgingSession && context.judgingSession == true };

// allow DoeFamily to view DoePhotos
permit (
  principal in PhotoApp::UserGroup::"DoeFamily",
  action == PhotoApp::Action::"viewPhoto",
  resource in PhotoApp::Album::"DoePhotos"
);

// allow JohnDoe to view JaneVacation
permit (
  principal in PhotoApp::User::"JohnDoe",
  action == PhotoApp::Action::"viewPhoto",
  resource in PhotoApp::Album::"JaneVacation"
);