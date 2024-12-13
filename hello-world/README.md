# Ratatui Hello World template

The hello world template will create the following project structure:

```text
.github/       -> GitHub workflows and dependabot configuration
src/
├── main.rs    -> entry-point and application logic
```

## Design choices

Terminal setup is done in the main function. Application logic is handled in the run function. This
makes it easier to handle panics and errors correctly in the main function.

We use [color-eyre](https://docs.rs/color-eyre/latest/color_eyre/) for simplifying any errors that
need to be reported to the console.
