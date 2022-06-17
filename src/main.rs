//mod btree;
//use btree::*;

fn main() {
    println!("{:?}", longest_substring("abcddabccabcdefgaa"));
}

fn longest_substring(string: &str) -> Vec<char> {

    let mut substring = Vec::<char>::new();
    let mut longest_substring = Vec::<char>::new();

    for c in string.chars() {
        let current_char = Some(c);
        if !substring.contains(&current_char.unwrap()) {
            substring.push(c);
            
        }
        else {
            //println!("\x1b[0;31m{:?}\t\t\t{:?}\x1b[0m", substring, longest_substring);
            if substring.len() > longest_substring.len() {
                longest_substring = substring.clone();
                substring = Vec::<char>::new();
            }
        }
    }

    if substring > longest_substring {
        return substring;
    }
    else {
        return longest_substring
    }
}

fn _x() {
    //     const TITLE_STR: &str = r"                    __          ___            __
    //     _______  _______/ /_   _____/ (_)__  ____  / /_
    //    / ___/ / / / ___/ __/  / ___/ / / _ \/ __ \/ __/
    //   / /  / /_/ (__  ) /_   / /__/ / /  __/ / / / /_
    //  /_/   \__,_/____/\__/   \___/_/_/\___/_/ /_/\__/
    //                                                    ";

    //     println!("{} \n", TITLE_STR);

    //     let tree_head = Node::new(1);
    //     let tree = Tree::new(tree_head);
    //     tree.print()
}
