# WASM Examples for Debashc

This directory contains comprehensive WASM examples for testing shell script conversion to multiple programming languages.

## Overview

The WASM examples provide an interactive web interface for testing the Debashc compiler's ability to convert shell scripts to various target languages including Perl, Rust, Python, Lua, C, JavaScript, English, French, Batch, and PowerShell.

## Available Example Pages

### 1. Comprehensive Examples (`comprehensive-examples.html`)
- **Purpose**: Complete testing interface with all examples organized by category
- **Features**: 
  - Sidebar navigation with categorized examples
  - Interactive testing panel
  - All language generators supported
  - Real-time conversion results
- **Best for**: Comprehensive testing and exploration of all features

### 2. New Split Examples Test (`split-examples-test.html`)
- **Purpose**: Focused testing of the new split examples
- **Features**:
  - Inline shell code display
  - Immediate conversion results
  - Card-based layout for easy comparison
  - Core language generators (Perl, Rust)
- **Best for**: Testing specific shell script features and syntax

### 3. Main Interface (`index.html`)
- **Purpose**: Primary WASM interface with categorized examples
- **Features**:
  - All language generators
  - Categorized example system
  - Real-time input processing
- **Best for**: General use and testing

### 4. Examples Index (`examples-index.html`)
- **Purpose**: Navigation hub for all WASM example pages
- **Features**:
  - Overview of all available pages
  - Quick access to different testing interfaces
  - Feature descriptions for each page
- **Best for**: Finding the right testing interface for your needs

### 5. Specialized Test Pages
- **Simple Test** (`simple-test.html`): Basic WASM functionality verification
- **LS Command Test** (`test-wasm-ls.html`): Specific command parsing test
- **Minimal Test** (`minimal-test.html`): Minimal interface for debugging
- **Debug WASM** (`debug-wasm.html`): Debug-focused interface

## New Split Examples Included

The new examples showcase advanced shell script features:

### ANSI-C Quoting
- `ansi_quoting_basic.sh` - Basic escape sequences
- `ansi_quoting_escape.sh` - Escape sequence handling
- `ansi_quoting_unicode.sh` - Unicode and hex support
- `ansi_quoting_practical.sh` - Real-world formatting

### Parameter Expansion
- `parameter_expansion_case.sh` - Case modification (uppercase/lowercase)
- `parameter_expansion_advanced.sh` - Advanced operations
- `parameter_expansion_more.sh` - Additional operations
- `parameter_expansion_defaults.sh` - Default value handling

### Arrays
- `arrays_indexed.sh` - Indexed array operations
- `arrays_associative.sh` - Associative array with key-value pairs

### Control Flow
- `control_flow_if.sh` - If statement examples
- `control_flow_loops.sh` - Loop constructs
- `control_flow_function.sh` - Function definitions

### Brace Expansion
- `brace_expansion_basic.sh` - Basic brace patterns
- `brace_expansion_advanced.sh` - Advanced sequences
- `brace_expansion_practical.sh` - File operation examples

### Pattern Matching
- `pattern_matching_basic.sh` - Basic glob patterns
- `pattern_matching_extglob.sh` - Extended glob support
- `pattern_matching_nocase.sh` - Case-insensitive matching

### Process Substitution
- `process_substitution_here.sh` - Here-string operations
- `process_substitution_comm.sh` - Comm command usage
- `process_substitution_mapfile.sh` - Mapfile operations
- `process_substitution_advanced.sh` - Complex examples

## Usage Instructions

### Getting Started
1. **Build WASM**: Ensure the WASM module is built and available in `pkg/`
2. **Open Examples**: Start with `examples-index.html` to see all options
3. **Choose Interface**: Select the testing interface that best fits your needs
4. **Test Examples**: Use the interactive buttons to test different operations

### Testing Operations
- **Tokenize**: Convert shell script to tokens
- **Parse**: Generate Abstract Syntax Tree (AST)
- **To Perl**: Convert to Perl code
- **To Rust**: Convert to Rust code
- **Experimental**: Test other language generators

### Example Workflow
1. Select an example from the sidebar/category
2. Choose an operation (e.g., "To Perl")
3. View the conversion result in the output area
4. Try different examples to test various shell features

## Technical Details

### WASM Module
- **File**: `pkg/debashl.js`
- **Functions**: `lex()`, `parse()`, `to_perl()`, `to_rust()`, etc.
- **Examples Data**: `examples_json()` function provides all example content

### Browser Compatibility
- Modern browsers with ES6 module support
- WASM support required
- Tested on Chrome, Firefox, Safari, Edge

### Performance
- Real-time conversion with minimal latency
- Efficient tokenization and parsing
- Optimized for interactive use

## Development and Testing

### Adding New Examples
1. Add shell script to `examples/` directory
2. Ensure it's included in the examples data
3. Test with WASM interface
4. Update documentation if needed

### Debugging
- Use browser developer tools for console output
- Check WASM initialization status
- Verify example data loading
- Test individual operations

### Customization
- Modify CSS for different styling
- Add new language generators
- Extend example categories
- Customize testing workflows

## Troubleshooting

### Common Issues
- **WASM not initialized**: Wait for initialization to complete
- **Examples not loading**: Check `examples_json()` function
- **Conversion errors**: Verify shell script syntax
- **Performance issues**: Check browser WASM support

### Debug Steps
1. Check browser console for errors
2. Verify WASM module loading
3. Test with simple examples first
4. Check network requests for WASM files

## Future Enhancements

### Planned Features
- More language generators
- Enhanced error reporting
- Performance optimizations
- Additional example categories
- Export functionality for results

### Contributing
- Add new shell script examples
- Improve language generators
- Enhance user interface
- Optimize WASM performance
- Add new testing features

## Support

For issues or questions:
- Check browser console for error messages
- Verify WASM module compilation
- Test with known working examples
- Review example syntax and structure

---

**Note**: These examples demonstrate the full capabilities of the Debashc compiler and provide comprehensive testing of shell script conversion features.
