# Features

- **Cross-platform**: Works on Windows, macOS, and Linux.
- **High Performance**: Optimized for speed and efficiency.
- **User-friendly Interface**: Intuitive design for easy navigation.
- **Customizable**: Flexible settings to suit your needs.
- **CLI/TUI**: Command-line and terminal-based interfaces available.
- **Advanced Imports**: Supports importing apis from straight up from your project.

## REST Features

- **Methods**
  - GET
  - POST
  - PUT
  - PATCH
  - DELETE
  - OPTIONS (in future)
  - HEAD (in future)
  - TRACE (in future)
  - CONNECT (in future)
- **Params**
- **Headers**
- **Authentication**
  - Basic auth
  - Bearer token
  - Digest
  - JWT
  - OAuth1-2
  - API Key
- **Body**
  - form-data
  - x-www-form-urlencoded
  - raw
    - Text
    - JavaScript
    - JSON
    - HTML
    - XML
    - **Note:** Support for beautifying
  - GraphQL
    - **Note:** Auto-Fetching of GraphQL schema in future
- **Scripts** (JavaScript)
  - Pre-request scripts
  - Post-request scripts

## Collections

- **Organization**
  - Folders
  - Subfolders
- **Import/Export**
  - Postman
  - Insomnia
  - Swagger/OpenAPI
- **Environment Variables**
  - Global
  - Collection-specific
  - Environment-specific
- **Authorization**
  - Inherit from collection
  - Override at request level
- **Runs**
  - Single request
  - Collection run
  - Data-driven testing (in future)
  - Results reporting
  - Result Visualization (in future)
    - Charts
    - Graphs
    - Custom dashboards
    - Performance metrics

## Import/Export

- **Formats**
  - Postman
  - Insomnia
  - Swagger/OpenAPI

## Account Backup and Sync

**Note:** Currently not implemented, but can think of something in future


## Advanced Imports

Toss can import APIs straight from your project controllers or routes. It should be done by running a cli command with the path to your project.

- Search for controllers/routes files throughout the project.
- Extract API endpoints and metadata from the files.
- Create a Toss Collection from the imported APIs.
- Toss Collection should have structure if the project has a defined structure.(e.g. if the project has multiple controllers/routes files, they should be grouped into folders in the Toss Collection)
- Supported Frameworks: Spring, ASP.NET, Ruby on Rails, Next.js, Express.js, Django, Flask, FastAPI, Laravel, Quarkus
  - Future frameworks: React, Vue.js, Svelte, Angular
- Only REST APIs are supported for now.


- Needs to be free
- Needs to be secure
- Needs to be cross-platform

## Environment Variables
- This section will contain and store all the environment variables for the collection.
- Based on which collection is selected, or which api/folder is selected, the environment variables will be loaded into the section.
- Users should be able to add, edit, and delete environment variables from this section.
- Environment variables should be stored in a persistent manner, so they are not lost when the user closes the application.
- Environment variable values should be masked or hidden from view with a shortcut key to toggle visibility.
- If Evnvironment variable is not implemented yet, create a plan to implement it, and then implement it.

## Smart Environment Variables Creation
- Add a feature to allow users to create smart environment variables that can be used across collections and requests.
- When a collection's requests all have the similar base URL, the user should be able to create a smart environment variable as baseURL that should replace the base URL in all requests in that collection to come from this smart environment variable.
- This baseURL env creation can be done by user while on the collection section by pressing a shortcut key. (:env create)

## Snippets
- Snippets can be used to quickly insert most commonly used inputs into relevent fields
- Some of the good examples of where snippets could be useful are adding localhost url, http headers, http/https request methods, and commonly used paths from other requests in collection.

currently, the project parser only parses the urls. but i want to add more funtionality to it. for post, put patch methods, if the method contains an object defined within the project, it should make a body oject for it in raw-json format. plan it and then implement it.
