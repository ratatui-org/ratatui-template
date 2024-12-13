default:
    @just --list

generate-all:
    just generate-component
    just generate-hello-world
    just generate-simple
    just generate-simple-async

# The purpose of these targets is to make it easy to make changes to the templates and then
# regenerate the generated projects and view the expected changes in a git diff.

generate-component:
    rm -rv component-generated
    cargo generate --path ./component \
        --name component-generated \
        --define project-description="An example generated using the component template" \
        --define use-gitserver=false

generate-hello-world:
    rm -rv hello-world-generated
    cargo generate --path ./hello-world --name hello-world-generated

generate-simple:
    rm -rv simple-generated
    cargo generate --path ./simple --name simple-generated

generate-simple-async:
    rm -rv simple-async-generated
    cargo generate --path ./simple-async --name simple-async-generated
