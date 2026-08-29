import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

// /admin/users -> dashboard with the "Membres" (all) tab active.
// BUG-5-USERS fix: previously /admin/users returned 404.
export const load: PageLoad = () => {
  throw redirect(307, '/admin?tab=membres');
};
