use std::collections::HashMap;

pub fn get_task_command_map() -> HashMap<String, String> {
    let mut task_command_map = HashMap::new();
    task_command_map.insert("init".to_string(), "composer init".to_string());
    task_command_map.insert("install".to_string(), "composer install".to_string());
    task_command_map.insert("compile".to_string(), "composer check-platform-reqs".to_string());
    task_command_map.insert("build".to_string(), "composer run-script build".to_string());
    task_command_map.insert("test".to_string(), "composer run-script test".to_string());
    task_command_map.insert("deps".to_string(), "composer depends".to_string());
    task_command_map.insert("doc".to_string(), "composer doc".to_string());
    task_command_map.insert("clean".to_string(), "composer clear-cache".to_string());
    task_command_map.insert("outdated".to_string(), "composer outdated".to_string());
    task_command_map.insert("update".to_string(), "composer update".to_string());
    task_command_map
}
