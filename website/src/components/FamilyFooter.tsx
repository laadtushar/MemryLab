import { CURRENT_APP_ID, FAMILY_APPS, FAMILY_BUILDER, FAMILY_STUDIO } from "@/lib/family";

/**
 * The "LabyNator family" block: the studio, the sibling apps and the builder.
 * Plain links, same tab, no nofollow (these are the owner's own sites).
 */
export default function FamilyFooter({ className = "" }: { className?: string }) {
  const siblings = FAMILY_APPS.filter((app) => app.id !== CURRENT_APP_ID);
  return (
    <section aria-label="LabyNator family" className={`text-xs text-zinc-500 ${className}`}>
      <div className="flex flex-col gap-3 md:flex-row md:flex-wrap md:items-center md:gap-x-6 md:gap-y-2">
        <p>
          Part of{" "}
          <a href={FAMILY_STUDIO.url} className="font-semibold text-zinc-300 hover:text-white transition">
            {FAMILY_STUDIO.name}
          </a>
        </p>
        <ul className="flex flex-col gap-1.5 md:flex-row md:flex-wrap md:gap-x-5">
          {siblings.map((app) => (
            <li key={app.id}>
              <a href={app.url} className="text-zinc-400 hover:text-white transition">
                {`${app.name} \u2014 ${app.blurb}`}
              </a>
            </li>
          ))}
        </ul>
        <p>
          Built by{" "}
          <a href={FAMILY_BUILDER.url} className="font-semibold text-zinc-300 hover:text-white transition">
            {FAMILY_BUILDER.name}
          </a>
        </p>
      </div>
    </section>
  );
}
