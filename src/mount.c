#include <assert.h>
#include <ctype.h>
#include <dirent.h>
#include <fcntl.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

typedef struct
{
	unsigned char	BS_jmpBoot[3];
	unsigned char	oem[8];

	unsigned short	BPB_BytesPerSector; //sector_size
	unsigned char	BPB_SectorsPerCluster;
	unsigned short	BPB_RsvdSecCnt; //reserved sectors
	unsigned char	BPB_NumberofFATS;
	unsigned short	BPB_RootEntCnt; //root_dir_entries
	unsigned short	BPB_TotalSectorsShort;
	unsigned char	BPB_MediaDescriptor;
	unsigned short	BPB_FATSz16; //fat_size_sectors
	unsigned short	BPB_SectorsPerTrack;
	unsigned short	BPB_NumberOfHeads;
	unsigned int	BPB_HiddenSectors;
	unsigned int	BPB_TotalSectorsLong;


	unsigned int	BPB_FATSz32;
	unsigned short	BPB_ExtFlags;
	unsigned short	BPB_FSVer;
	unsigned int	BPB_RootCluster;


	unsigned short	BPB_FSInfo;
	unsigned short	BPB_BkBootSec;
	unsigned char	BPB_Reserved[12];
	unsigned char	BS_DrvNum;
	unsigned char	BS_Reserved1;
	unsigned char	BS_BootSig;
	unsigned int	BS_VolID; 
	unsigned char	BS_VolLab[11];
	unsigned char	BS_FilSysType[8];
} __attribute__((packed)) FAT32BootBlock;

typedef struct
{
	unsigned int current_cluster_number;
	unsigned char name[100];

	char currentpath[50][100];
	int current_cluster_path[50];
	int current;

} __attribute__((packed)) Environment;



FILE* imageFile;
FAT32BootBlock bootBlock;
Environment ENV;
int firstDataSector;



void addToEnvPath(int currentCluster, char* name);



int main(int argc, char* argv[]) {
	if (argc != 2) {
		fprintf(stderr, "Usage: %s <FAT32_ISO>\n", argv[0]);
		return 1;
	}

	char cmd[100];

	// Open image file
	char fileName[256];
	snprintf(fileName, sizeof(fileName), "./%s", argv[1]);

	imageFile = fopen(fileName, "rb+");
	fread(&bootBlock, sizeof(FAT32BootBlock), 1, imageFile);

	firstDataSector = bootBlock.BPB_RsvdSecCnt + (bootBlock.BPB_NumberofFATS * bootBlock.BPB_FATSz32);

	ENV.current = 0;
	char myname[] = "/";

	addToEnvPath(bootBlock.BPB_RootCluster, myname);
	
	while (1) {
		// char* user = getenv("USER");
		// char* machine = getenv("MACHINE");
		char* pwd = getenv("PWD");

		printf("%s%s", argv[1], pwd);
		for (int i = 0; i < ENV.current; i++) {
			printf("%s/", i == 0 ? "" : ENV.currentpath[i]);
		}
		printf("> ");

		fgets(cmd, sizeof(cmd), stdin);

		if (cmd[strlen(cmd) - 1] == '\n') {
			cmd[strlen(cmd) - 1] = '\0';
		}
		if (strcmp(cmd, "exit") == 0) {
			break;
		}
	}

	return 0;
}



void addToEnvPath(int currentCluster, char * name)
{
	ENV.current_cluster_number = currentCluster;
	strcpy((char *)ENV.name, name);
	strcpy(ENV.currentpath[ENV.current], name);

	ENV.current_cluster_path[ENV.current] = currentCluster;

	ENV.current++;
}