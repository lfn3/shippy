# shippy

### Running tests

By default we don't run the tests under the `gitlab_api_tests` feature. 
In order to run these, you have to supply an API token for Gitlab.com.
This needs to be passed in via the `GITLAB_API_TOKEN` env var when the `gitlab_api_tests`
feature is enabled in order to run these tests:

```shell script
GITLAB_API_TOKEN=ABC cargo test --features=gitlab_api_tests 
```