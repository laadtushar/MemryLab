import HomePage from "@/components/HomePage";
import { JsonLd } from "@/lib/jsonld";
import { pageMetadata } from "@/lib/seo";
import { AUTHOR, LATEST_RELEASE_URL, PUBLISHER, REPO_URL, SITE_DESCRIPTION, SITE_NAME, SITE_URL } from "@/lib/site";

export const metadata = pageMetadata({
  title: "MemryLab: Your Memory, Visualized | Local AI Memory App",
  description: SITE_DESCRIPTION,
  path: "/",
});

const softwareApplication = {
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  name: SITE_NAME,
  url: SITE_URL,
  description: SITE_DESCRIPTION,
  applicationCategory: "ProductivityApplication",
  operatingSystem: "Windows, macOS, Linux",
  downloadUrl: LATEST_RELEASE_URL,
  license: `${REPO_URL}/blob/master/LICENSE`,
  isAccessibleForFree: true,
  offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
  publisher: PUBLISHER,
  creator: PUBLISHER,
  author: AUTHOR,
};

export default function Page() {
  return (
    <>
      <JsonLd data={softwareApplication} />
      <HomePage />
    </>
  );
}
