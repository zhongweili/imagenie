import { GITHUB_REPO, ModelRelease } from '../config/models';

export async function getLatestModelRelease(modelName: string): Promise<ModelRelease> {
  const apiUrl = `https://api.github.com/repos/${GITHUB_REPO.owner}/${GITHUB_REPO.repo}/releases/tags/models-v0.1.0`;

  try {
    const response = await fetch(apiUrl);
    if (!response.ok) {
      throw new Error('Failed to fetch release info');
    }

    const releaseData = await response.json();

    // Find the model asset in the release
    const modelAsset = releaseData.assets.find(
      (asset: any) => asset.name.includes(modelName)
    );

    if (!modelAsset) {
      throw new Error('Model not found in latest release');
    }

    return {
      version: releaseData.tag_name,
      url: modelAsset.browser_download_url,
      fileName: modelAsset.name
    };
  } catch (error) {
    console.error('Error fetching release info:', error);
    throw error;
  }
}
