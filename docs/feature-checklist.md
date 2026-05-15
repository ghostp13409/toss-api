# Feature Checklist

## Cross-Platform
- [x] Linux
- [x] Windows
- [ ] macOS (not tested)

## Rest

- **Methods**
  - [x] GET
  - [x] POST
  - [x] PUT
  - [x] PATCH
  - [x] DELETE
  - [ ] OPTIONS (in future)
  - [ ] HEAD (in future)
  - [ ] TRACE (in future)
  - [ ] CONNECT (in future)
- [x] **Params**
- [x] **Headers**
- [ ] **Authentication**
  - [x] Basic auth
  - [x] Bearer token
  - [x] API Key
  - [ ] Digest
  - [ ] JWT
  - [ ] OAuth1-2
- **Body**
  - [x] form-data
  - [x] x-www-form-urlencoded
  - [x] raw
    - Text
    - JavaScript
    - JSON
    - HTML
    - XML
  - [x] Linter, Beautifier, Syntax highlighting
  - [ ] GraphQL
    - [ ] Auto-Fetching of GraphQL (in future)
- [ ] **Scripts** (JavaScript)
  - [ ] Pre-request scripts
  - [ ] Post-request scripts

## Collections
- [x] **Organization**
  - [x] Folders
  - [x] Subfolders
- [ ] **Import**
  - [x] Postman
  - [ ] Insomnia
  - [ ] Swagger/OpenAPI
- [ ] **Export**
  - [ ] Postman
  - [ ] Insomnia
  - [ ] Swagger/OpenAPI
- [ ] **Environment Variables**
  - [ ] Global
  - [x] Collection-specific
  - [ ] Environment-specific
- [ ] **Authorization**
  - [ ] Inherit from collection
  - [ ] Override at request level
- [ ] **Runs**
  - [ ] Single request
  - [ ] Collection run
  - [ ] Data-driven testing (in future)
  - [ ] Results reporting
  - [ ] Result Visualization (in future)
    - [ ] Charts
    - [ ] Graphs
    - [ ] Custom dashboards
    - [ ] Performance metrics

## Advanced Imports

- [ ] **Frameworks**
  - [x] Spring
  - [x] ASP.NET
  - [x] Ruby on Rails
  - [x] Next.js
  - [x] Express.js
  - [x] Django
  - [x] Flask
  - [x] FastAPI
  - [x] Laravel
  - [x] Quarkus
  - [ ] React
  - [ ] Vue.js
  - [ ] Svelte
  - [ ] Angular
- Toss can import APIs straight from your project controllers or routes. It should be done by running a cli command with the path to your project.

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
