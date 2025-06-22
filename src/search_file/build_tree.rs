
use std::path::PathBuf;

use crate::search_file;
use crate::search_file::SingleFileInformations;

struct FolderInformations {
    foldname : String,
    father: Option<usize>,
    childs: Vec<usize>,
}

fn build_tree(files: &Vec<SingleFileInformations>) -> Vec<FolderInformations> {
    let mut tree: Vec<FolderInformations> = Vec::new();
    let mut current_path = PathBuf::new();
    let mut current:usize=0;
    tree.push(FolderInformations { foldname: String::new(), father: None, childs: Vec::new() });
    for file in files {
        let mut file_path = file.path.clone();
        while match file_path.strip_prefix(&current_path) {
            Err(e) => true,
            Ok(stripped) => false,
        } {
            current = tree[current].father.unwrap_or(0);
            current_path.pop();
        }
        let additional_path = file_path.strip_prefix(&current_path)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();
        // split the path by '\'
        let parts: Vec<&str> = additional_path.split(std::path::MAIN_SEPARATOR).collect();
        for part in parts {
            if part.is_empty() {
                continue; // Skip empty parts
            }
            let mut found = false;
            for (i, folder) in tree[current].childs.iter().enumerate() {
                if tree[*folder].foldname == part {
                    current = *folder;
                    found = true;
                    break;
                }
            }
            if !found {
                tree.push(FolderInformations {
                    foldname: part.to_string(),
                    father: Some(current),
                    childs: Vec::new(),
                });
                let mut new_folder_index:usize = (tree.len() as usize) - 1;
                tree[current].childs.push(new_folder_index);
                current = new_folder_index;
            }
        }
        current_path = file_path.clone();
    }
    tree
}

fn print_tree(tree: &Vec<FolderInformations>) -> String {
    let mut output = String::new();
    output.push_str("Tree structure:\n");
    let mut depth : Vec<u32> = vec![0; tree.len()];
    let mut ended : Vec<bool> = vec![false; tree.len()];
    
    for (id, folder) in tree.iter().enumerate() {
        if id == 0 {
            depth[id] = 0;
            continue;
            // skip the root folder
        }
        depth[id] = depth[folder.father.unwrap()] + 1;
        // println!("id:{}, depth: {}", id, depth[id]);
        for i in 1..depth[id] {
            if i == depth[id] - 1 {
                if tree[folder.father.unwrap()].childs.last() == Some(&id) {
                    output.push_str(" └──");
                    ended[(depth[id]-1) as usize] = true;
                } else {
                    output.push_str(" ├──");
                }
            } else {
                if(ended[i as usize]) {
                    output.push_str("    ");
                } else {
                    output.push_str(" │  ");
                }
            }
        }
        if folder.childs.is_empty() {
            output.push_str(&format!("{}\n", folder.foldname));
        } else {
            output.push_str(&format!("{} ({} child(s))\n", folder.foldname, folder.childs.len()));
        }
    }
    output
}

pub fn get_tree(files: &Vec<SingleFileInformations>) -> String {
    let tree: Vec<FolderInformations> = build_tree(&files);
    let output = print_tree(&tree);
    output
    // for (id, folder) in tree.iter().enumerate() {
    //     println!("id:{}, name: {}", id, folder.foldname);
    //     for child in &folder.childs {
    //         println!("    child id:{}, name: {}", child, tree[*child].foldname);
    //     }
    // }
}