# Blag

A blog system in rust.

## Templates

List of the required templates to run the site.

- list
- post
- page

It technically does not matter how those templates are created, just that they are.

I may add something to `post` and `page` to allow you to set your own template.

## Important Notes

Before doing anything with the template data/object ensure that all data has been collected.

## Available API Routes

The following API routes are available:

- `/post_list` (GET)
- `/post_get/{slug}` (GET)
- `/post_update` (POST)
- `/post_delete` (POST)
- `/page_list` (GET)
- `/page_get/{slug}` (GET)
- `/page_update/{slug}` (POST)
- `/page_delete` (POST)

All API routes require authentication via the `Auth` struct.

All API routes return JSON responses.

## Authentication

The `Auth` struct requires a bearer token for authentication.
