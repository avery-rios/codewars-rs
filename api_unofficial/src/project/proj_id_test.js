App.setup({
  env: "production",
  currentUser,
  data: JSON.parse(""),
  routes: {
    user_profile: "/users/example",
    user_stars: "/user/stars",
    star_code_challenge: "/users/stars/%7Bid%7D",
    mark_notifications_read: "/users/notifications/mark_read",
    unread_popup_notifications: "/users/notifications/unread_popups",
    collections: "/api/v1/collections",
    collection_code_challenge:
      "/api/v1/collections/%7BcollectionId%7D/code_challenges/%7Bid%7D",
    session: "/kata/projects/aaaaaaaaaaaaaaaaaaaaaaaa/%7Blanguage%7D/session",
    notify:
      "/api/v1/code-challenges/projects/aaaaaaaaaaaaaaaaaaaaaaaa/solutions/%7BsolutionId%7D/notify",
    finalize:
      "/api/v1/code-challenges/projects/aaaaaaaaaaaaaaaaaaaaaaaa/solutions/%7BsolutionId%7D/finalize",
    skip: "/kata/projects/aaaaaaaaaaaaaaaaaaaaaaaa/skip",
    report: "/kata/000000000000000000000000",
    comments: "/kata/000000000000000000000000/discuss/rust",
    solutions: "/kata/000000000000000000000000/solutions/%7Blanguage%7D",
    editor: "/kata/000000000000000000000000/edit/%7Blanguage%7D",
    forfeit: "/kata/000000000000000000000000/solutions?show-solutions=1",
  },
  pageControllerName: "CodeChallenges.PlayController",
});
