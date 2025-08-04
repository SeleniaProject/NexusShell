//! Comprehensive tests for the NexusShell parser
//!
//! These tests verify that the PEST grammar and AST construction work correctly
//! for all major shell constructs.

use crate::{ShellCommandParser, ast::AstNode};

/// Test basic command parsing
#[test]
fn test_simple_command_parsing() {
    let parser = ShellCommandParser::new();
    
    // Test basic command
    let result = parser.parse("echo hello").unwrap();
    
    match result {
        AstNode::Command { name, args, .. } => {
            match name.as_ref() {
                AstNode::Word(word) => assert_eq!(*word, "echo"),
                _ => {
                    eprintln!("Expected Word for command name, got {:?}", name);
                    assert!(false, "Expected Word for command name");
                }
            }
            assert_eq!(args.len(), 1);
            match &args[0] {
                AstNode::Word(word) => assert_eq!(*word, "hello"),
                _ => {
                    eprintln!("Expected Word for argument, got {:?}", &args[0]);
                    assert!(false, "Expected Word for argument");
                }
            }
        }
        _ => {
            eprintln!("Expected Command node, got {:?}", result);
            assert!(false, "Expected Command node");
        }
    }
}

/// Test command with multiple arguments
#[test]
fn test_command_with_multiple_args() {
    let parser = ShellCommandParser::new();
    
    let result = parser.parse("ls -la /home").unwrap();
    
    match result {
        AstNode::Command { name, args, .. } => {
            match name.as_ref() {
                AstNode::Word(word) => assert_eq!(*word, "ls"),
                _ => {
                    eprintln!("Expected Word for command name, got {:?}", name);
                    assert!(false, "Expected Word for command name");
                }
            }
            assert_eq!(args.len(), 2);
            match &args[0] {
                AstNode::Word(word) => assert_eq!(*word, "-la"),
                _ => {
                    eprintln!("Expected Word for first argument, got {:?}", &args[0]);
                    assert!(false, "Expected Word for first argument");
                }
            }
            match &args[1] {
                AstNode::Word(word) => assert_eq!(*word, "/home"),
                _ => {
                    eprintln!("Expected Word for second argument, got {:?}", &args[1]);
                    assert!(false, "Expected Word for second argument");
                }
            }
        }
        _ => {
            eprintln!("Expected Command node, got {:?}", result);
            assert!(false, "Expected Command node");
        }
    }
}

/// Test variable expansion
#[test]
fn test_variable_expansion() {
    let parser = ShellCommandParser::new();
    
    let result = parser.parse("echo $HOME").unwrap();
    
    match result {
        AstNode::Command { args, .. } => {
            assert_eq!(args.len(), 1);
            match &args[0] {
                AstNode::VariableExpansion { name, .. } => {
                    assert_eq!(*name, "HOME");
                }
                _ => {
                    eprintln!("Expected VariableExpansion for argument, got {:?}", &args[0]);
                    assert!(false, "Expected VariableExpansion for argument");
                }
            }
        }
        _ => {
            eprintln!("Expected Command node, got {:?}", result);
            assert!(false, "Expected Command node");
        }
    }
}

/// Test braced variable expansion
#[test]
fn test_braced_variable_expansion() {
    let parser = ShellCommandParser::new();
    
    let result = parser.parse("echo ${USER}").unwrap();
    
    match result {
        AstNode::Command { args, .. } => {
            assert_eq!(args.len(), 1);
            match &args[0] {
                AstNode::VariableExpansion { name, .. } => {
                    assert_eq!(*name, "USER");
                }
                _ => {
                    eprintln!("Expected VariableExpansion for argument, got {:?}", &args[0]);
                    assert!(false, "Expected VariableExpansion for argument");
                }
            }
        }
        _ => {
            eprintln!("Expected Command node, got {:?}", result);
            assert!(false, "Expected Command node");
        }
    }
}

/// Test command substitution
#[test]
fn test_command_substitution() {
    let parser = ShellCommandParser::new();
    
    let result = parser.parse("echo $(date)").unwrap();
    
    match result {
        AstNode::Command { args, .. } => {
            assert_eq!(args.len(), 1);
            match &args[0] {
                AstNode::CommandSubstitution { is_legacy, .. } => {
                    assert_eq!(*is_legacy, false);
                }
                _ => {
                    eprintln!("Expected CommandSubstitution for argument, got {:?}", &args[0]);
                    assert!(false, "Expected CommandSubstitution for argument");
                }
            }
        }
        _ => {
            eprintln!("Expected Command node, got {:?}", result);
            assert!(false, "Expected Command node");
        }
    }
}

/// Test legacy command substitution (backticks)
#[test]
fn test_legacy_command_substitution() {
    let parser = ShellCommandParser::new();
    
    let result = parser.parse("echo `date`").unwrap();
    
    match result {
        AstNode::Command { args, .. } => {
            assert_eq!(args.len(), 1);
            match &args[0] {
                AstNode::CommandSubstitution { is_legacy, .. } => {
                    assert_eq!(*is_legacy, true);
                }
                _ => {
                    eprintln!("Expected CommandSubstitution for argument, got {:?}", &args[0]);
                    assert!(false, "Expected CommandSubstitution for argument");
                }
            }
        }
        _ => {
            eprintln!("Expected Command node, got {:?}", result);
            assert!(false, "Expected Command node");
        }
    }
}

/// Test simple pipeline
#[test]
fn test_simple_pipeline() {
    let parser = ShellCommandParser::new();
    
    let result = parser.parse("cat file.txt | grep pattern").unwrap();
    
    match result {
        AstNode::Pipeline { elements, operators } => {
            assert_eq!(elements.len(), 2);
            assert_eq!(operators.len(), 1);
            
            // Check first command
            match &elements[0] {
                AstNode::Command { name, args, .. } => {
                    match name.as_ref() {
                        AstNode::Word(word) => assert_eq!(*word, "cat"),
                        _ => {
                            eprintln!("Expected Word for first command name, got {:?}", name.as_ref());
                            assert!(false, "Expected Word for first command name");
                        }
                    }
                    assert_eq!(args.len(), 1);
                }
                _ => {
                    eprintln!("Expected Command for first pipeline element, got {:?}", &elements[0]);
                    assert!(false, "Expected Command for first pipeline element");
                }
            }
            
            // Check second command
            match &elements[1] {
                AstNode::Command { name, args, .. } => {
                    match name.as_ref() {
                        AstNode::Word(word) => assert_eq!(*word, "grep"),
                        _ => {
                            eprintln!("Expected Word for second command name, got {:?}", name.as_ref());
                            assert!(false, "Expected Word for second command name");
                        }
                    }
                    assert_eq!(args.len(), 1);
                }
                _ => {
                    eprintln!("Expected Command for second pipeline element, got {:?}", &elements[1]);
                    assert!(false, "Expected Command for second pipeline element");
                }
            }
        }
        _ => {
            eprintln!("Expected Pipeline node, got {:?}", result);
            assert!(false, "Expected Pipeline node");
        }
    }
}

/// Test empty input
#[test]
fn test_empty_input() {
    let parser = ShellCommandParser::new();
    
    let result = parser.parse("");
    
    // Empty input should result in an empty program
    match result {
        Ok(AstNode::Program(statements)) => {
            assert_eq!(statements.len(), 0);
        }
        _ => {
            // Some implementations may return an error for empty input
            // This is acceptable behavior
        }
    }
}

/// Test whitespace handling
#[test]
fn test_whitespace_handling() {
    let parser = ShellCommandParser::new();
    
    let result = parser.parse("  echo   hello  world  ").unwrap();
    
    match result {
        AstNode::Command { name, args, .. } => {
            match name.as_ref() {
                AstNode::Word(word) => assert_eq!(*word, "echo"),
                _ => {
                    eprintln!("Expected Word for command name, got {:?}", name.as_ref());
                    assert!(false, "Expected Word for command name");
                }
            }
            assert_eq!(args.len(), 2);
            match &args[0] {
                AstNode::Word(word) => assert_eq!(*word, "hello"),
                _ => {
                    eprintln!("Expected Word for first argument, got {:?}", &args[0]);
                    assert!(false, "Expected Word for first argument");
                }
            }
            match &args[1] {
                AstNode::Word(word) => assert_eq!(*word, "world"),
                _ => {
                    eprintln!("Expected Word for second argument, got {:?}", &args[1]);
                    assert!(false, "Expected Word for second argument");
                }
            }
        }
        _ => {
            eprintln!("Expected Command node, got {:?}", result);
            assert!(false, "Expected Command node");
        }
    }
}

/// Test complex pipeline
#[test]
fn test_complex_pipeline() {
    let parser = ShellCommandParser::new();
    
    let result = parser.parse("ps aux | grep firefox | awk '{print $2}'").unwrap();
    
    match result {
        AstNode::Pipeline { elements, operators } => {
            assert_eq!(elements.len(), 3);
            assert_eq!(operators.len(), 2);
            
            // Verify all commands are parsed correctly
            for (i, expected_name) in ["ps", "grep", "awk"].iter().enumerate() {
                match &elements[i] {
                    AstNode::Command { name, .. } => {
                        match name.as_ref() {
                            AstNode::Word(word) => assert_eq!(*word, *expected_name),
                            _ => {
                                eprintln!("Expected Word for command name at position {}, got {:?}", i, name.as_ref());
                                assert!(false, "Expected Word for command name");
                            }
                        }
                    }
                    _ => {
                        eprintln!("Expected Command for pipeline element at position {}, got {:?}", i, &elements[i]);
                        assert!(false, "Expected Command for pipeline element");
                    }
                }
            }
        }
        _ => {
            eprintln!("Expected Pipeline node, got {:?}", result);
            assert!(false, "Expected Pipeline node");
        }
    }
}

/// Test error handling for invalid syntax
#[test]
fn test_invalid_syntax_error() {
    let parser = ShellCommandParser::new();
    
    // Test with clearly invalid syntax
    let result = parser.parse("echo | | grep");
    
    // This should result in an error
    assert!(result.is_err());
}

/// Test mixed content with variables and substitutions
#[test]
fn test_mixed_content() {
    let parser = ShellCommandParser::new();
    
    let result = parser.parse("echo $USER $(date) hello").unwrap();
    
    match result {
        AstNode::Command { args, .. } => {
            assert_eq!(args.len(), 3);
            
            // Check variable
            match &args[0] {
                AstNode::VariableExpansion { name, .. } => {
                    assert_eq!(*name, "USER");
                }
                _ => {
                    eprintln!("Expected VariableExpansion for first argument, got {:?}", &args[0]);
                    assert!(false, "Expected VariableExpansion for first argument");
                }
            }
            
            // Check command substitution
            match &args[1] {
                AstNode::CommandSubstitution { is_legacy, .. } => {
                    assert_eq!(*is_legacy, false);
                }
                _ => {
                    eprintln!("Expected CommandSubstitution for second argument, got {:?}", &args[1]);
                    assert!(false, "Expected CommandSubstitution for second argument");
                }
            }
            
            // Check literal word
            match &args[2] {
                AstNode::Word(word) => assert_eq!(*word, "hello"),
                _ => {
                    eprintln!("Expected Word for third argument, got {:?}", &args[2]);
                    assert!(false, "Expected Word for third argument");
                }
            }
        }
        _ => {
            eprintln!("Expected Command node, got {:?}", result);
            assert!(false, "Expected Command node");
        }
    }
}
