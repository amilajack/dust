use std::collections::HashMap;
use std::collections::HashSet;

use ignore::WalkBuilder;

use std::path::Path;
use std::path::PathBuf;

use lib::Node;
mod platform;
use self::platform::*;

pub fn get_dir_tree(filenames: &Vec<&str>, apparent_size: bool) -> (bool, HashMap<String, u64>) {
    let mut permissions = true;
    let mut data = HashMap::new();
    for &b in filenames {
        match Path::new(&b).canonicalize() {
            Ok(p) => {
                let hp = examine_dir(&p, apparent_size, &mut data);
                permissions = permissions && hp;
            },
            Err(_) => {}
        }
    }
    //println!("{:?}", data);
    (permissions, data)
}

use ignore::WalkState;
use std::sync::{Arc, Mutex};

fn examine_dir(
    top_dir: &PathBuf,
    apparent_size: bool,
    data : &mut HashMap<String, u64>
) -> (bool) {
    let mut have_permission = true;
    let paths = Arc::new(Mutex::new(HashMap::new()));
    let inodes = Arc::new(Mutex::new(HashSet::new()));

    WalkBuilder::new(top_dir).standard_filters(false).threads(10).build_parallel().run(
        ||
        {
            let paths = paths.clone();
            let inodes = inodes.clone();
            let top_dir2 = top_dir.clone();
            Box::new(move |r| {
                match r {
                    Ok(de) => {
                        let maybe_size_and_inode = get_metadata(&de, apparent_size);

                        match maybe_size_and_inode {
                            Some((size, maybe_inode)) => {
                                if !apparent_size {
                                    if let Some(inode_dev_pair) = maybe_inode {
                                        let mut inodes = inodes.lock().unwrap();
                                        if inodes.contains(&inode_dev_pair) {
                                            return WalkState::Continue;
                                        }
                                        inodes.insert(inode_dev_pair);
                                    }
                                }

                                let mut paths = paths.lock().unwrap();
                                let mut e_path = de.path().to_path_buf();
                                while e_path != top_dir2 {
                                    let path_name = e_path.to_string_lossy().to_string();
                                    let s = paths.entry(path_name).or_insert(0);
                                    *s += size;
                                    e_path.pop();
                                }
                                let path_name = e_path.to_string_lossy().to_string();
                                let s = paths.entry(path_name).or_insert(0);
                                *s += size;

                            },
                            None => have_permission = false,
                        }
                    },
                    _ => {}
                };
                WalkState::Continue
            })
        }
    );
    let paths2 = paths.lock().unwrap();
    for (k,v) in paths2.iter() {
        data.insert(k.to_string(), *v);
    }
    (have_permission)
}
        /*match entry {
            Ok(e) => {
                //println!("{:?}", e.path());
                let maybe_size_and_inode = get_metadata(&e, apparent_size);

                match maybe_size_and_inode {
                    Some((size, maybe_inode)) => {
                        if !apparent_size {
                            if let Some(inode_dev_pair) = maybe_inode {
                                if inodes.contains(&inode_dev_pair) {
                                    continue;
                                }
                                inodes.insert(inode_dev_pair);
                            }
                        }
                        let mut e_path = e.path().to_path_buf();
                        while e_path != *top_dir {
                            let path_name = e_path.to_string_lossy().to_string();
                            let s = data.entry(path_name).or_insert(0);
                            *s += size;
                            e_path.pop();
                        }

                    },
                    None => have_permission = false,
                }
            },
            _ => {}
        }*/

// We start with a list of root directories - these must be the biggest folders
// We then repeadedly merge in the children of the biggest directory - Each iteration
// the next biggest directory's children are merged in.
pub fn find_big_ones<'a>(
    data: HashMap<String, u64>,
    max_to_show: usize
) -> Vec<Node> {

    let mut new_l: Vec<Node> = data.iter().map(|(a, b)| Node::new(a, *b, Vec::new())).collect();
    new_l.sort();
/*
    let mut new_l: Vec<&Node> = l.iter().map(|a| a).collect();
    new_l.sort();

    for processed_pointer in 0..max_to_show {
        if new_l.len() == processed_pointer {
            break;
        }
        // Must be a list of pointers into new_l otherwise b_list will go out of scope
        // when it is deallocated
        let mut b_list: Vec<&Node> = new_l[processed_pointer]
            .children()
            .iter()
            .map(|a| a)
            .collect();
        new_l.extend(b_list);
        new_l.sort();
    }
    */

    if new_l.len() > max_to_show {
        new_l[0..max_to_show + 1].to_vec()
    } else {
        new_l
    }
}
