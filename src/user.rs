pub fn is_root() -> bool {
    uzers::get_current_gid() == 0
}
